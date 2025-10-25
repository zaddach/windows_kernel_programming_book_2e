#![no_std]

use core::ffi::c_void;

use booster_common::ThreadData;
use wdk_sys::{
   ntddk::{IoCreateDevice, IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink, IofCompleteRequest, KeSetPriorityThread, ObfDereferenceObject, PsLookupThreadByThreadId}, DRIVER_OBJECT, FILE_DEVICE_UNKNOWN, HANDLE, IO_NO_INCREMENT, NTSTATUS, NT_SUCCESS, PCUNICODE_STRING, PDEVICE_OBJECT, PETHREAD, PIO_STACK_LOCATION, PIRP, STATUS_BUFFER_TOO_SMALL, STATUS_INVALID_PARAMETER, STATUS_SUCCESS, UNICODE_STRING
};
use wdk::println;
use wkpb_util::u;

#[cfg(not(test))]
extern crate wdk_panic;

#[cfg(not(test))]
use wdk_alloc::WdkAllocator;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WdkAllocator = WdkAllocator;

unsafe extern "C" {
   fn IoGetCurrentIrpStackLocation(irp: PIRP) -> PIO_STACK_LOCATION;
}

const DEVICE_NAME: UNICODE_STRING = u!(r"\Device\Booster");
const DEVICE_SYMLINK: UNICODE_STRING = u!(r"\??\Booster");

// SAFETY: "DriverEntry" is the required symbol name for Windows driver entry points.
// No other function in this compilation unit exports this name, preventing symbol conflicts.
#[unsafe(export_name = "DriverEntry")] // WDF expects a symbol with the name DriverEntry
pub unsafe extern "system" fn driver_entry(
   driver: &mut DRIVER_OBJECT,
   _registry_path: PCUNICODE_STRING,
) -> NTSTATUS {
   println!("Booster: DriverEntry");
   driver.DriverUnload = Some(booster_unload);
   driver.MajorFunction[wdk_sys::IRP_MJ_CREATE as usize] = Some(booster_create_close);
   driver.MajorFunction[wdk_sys::IRP_MJ_CLOSE as usize] = Some(booster_create_close);
   driver.MajorFunction[wdk_sys::IRP_MJ_WRITE as usize] = Some(booster_write);

   let mut device_object = PDEVICE_OBJECT::default();
   unsafe {
      let status = IoCreateDevice(
         driver,
         0,
         &DEVICE_NAME as *const _ as *mut _,
         FILE_DEVICE_UNKNOWN,
         0,
         false.into(),
         &mut device_object
      );

      if !NT_SUCCESS(status) {
         println!("Failed to create device object (0x{:08X})", status);
         return status;
      }

      let status = IoCreateSymbolicLink(&DEVICE_SYMLINK as *const _ as *mut _, &DEVICE_NAME as *const _ as *mut _);
      if !NT_SUCCESS(status) {
         println!("Failed to create symbolic link (0x{:08X})", status);
         IoDeleteDevice(device_object); // Important
         return status;
      }
   }

   STATUS_SUCCESS
}

unsafe extern "C" fn booster_unload(driver: *mut DRIVER_OBJECT) {
   println!("Booster: Driver unload");

   unsafe {
      let _ = IoDeleteSymbolicLink(&DEVICE_SYMLINK as *const _ as *mut _);
      if let Some(driver) = driver.as_ref() {
         let _ = IoDeleteDevice(driver.DeviceObject);
      }
   }
}

unsafe extern "C" fn booster_create_close(
   _device: *mut wdk_sys::DEVICE_OBJECT,
   irp: *mut wdk_sys::IRP,
) -> NTSTATUS {
   unsafe {
      if let Some(irp) = irp.as_mut() {
         irp.IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
         irp.IoStatus.Information = 0;
         IofCompleteRequest(irp, wdk_sys::IO_NO_INCREMENT as i8);
      }
   }
   STATUS_SUCCESS
}

unsafe extern "C" fn booster_write(
   _device: *mut wdk_sys::DEVICE_OBJECT,
   irp: *mut wdk_sys::IRP,
) -> NTSTATUS {
   let mut status = STATUS_SUCCESS;
   let mut information = 0;
   unsafe {
      if let Some(irp) = irp.as_mut() {
         let irp_sp = IoGetCurrentIrpStackLocation(irp);
         loop {
            if let Some(irp_sp) = irp_sp.as_ref() {
               if irp_sp.Parameters.Write.Length < core::mem::size_of::<ThreadData>() as u32 {
                  status = STATUS_BUFFER_TOO_SMALL;
                  break;
               }

               let data = irp.UserBuffer as *const ThreadData;
               if let Some(data) = data.as_ref() {
                  if data.priority < 1 || data.priority > 31 {
                     status = STATUS_INVALID_PARAMETER;
                     break;
                  }

                  let mut thread = PETHREAD::default();
                  status = PsLookupThreadByThreadId( data.thread_id as HANDLE, &mut thread);
                  if !NT_SUCCESS(status) {
                     break;
                  }

                  let old_priority = KeSetPriorityThread(thread, data.priority);
                  println!("Priority changed for thread {} from {} to {}", data.thread_id, old_priority, data.priority);

                  ObfDereferenceObject(thread as *mut c_void);
                  information = core::mem::size_of::<ThreadData>() as u64;
               }
               else {
                  status = STATUS_INVALID_PARAMETER;
                  break;
               }
            }

            break;
         }

         irp.IoStatus.__bindgen_anon_1.Status = status;
         irp.IoStatus.Information = information;
         IofCompleteRequest(irp, IO_NO_INCREMENT as i8);
      }
   }
   STATUS_SUCCESS
}
