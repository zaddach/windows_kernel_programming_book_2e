#![no_std]

use wdk::println;
use wdk_strings::u;
use wdk_sys::{
    DO_DIRECT_IO, DRIVER_OBJECT, FILE_DEVICE_UNKNOWN, IO_NO_INCREMENT, IRP_MJ_CLOSE, IRP_MJ_CREATE, IRP_MJ_DEVICE_CONTROL, IRP_MJ_WRITE, NT_SUCCESS, NTSTATUS, PCUNICODE_STRING, PDEVICE_OBJECT, STATUS_SUCCESS, UNICODE_STRING, STATUS_INVALID_BUFFER_SIZE, IRP_MJ_READ,
    ntddk::{
        IoCreateDevice, IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink,
        IofCompleteRequest,
        DbgPrint,
    },
};
use windows_drivers_util::IoGetCurrentIrpStackLocation;

#[cfg(not(test))]
extern crate wdk_panic;

#[cfg(not(test))]
use wdk_alloc::WdkAllocator;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WdkAllocator = WdkAllocator;

const DEVICE_NAME: UNICODE_STRING = u!(r"\Device\Zero");
const DEVICE_SYMLINK: UNICODE_STRING = u!(r"\??\Zero");
const DRIVER_PREFIX: &[u8] = b"Zero: ";

// SAFETY: "DriverEntry" is the required symbol name for Windows driver entry points.
// No other function in this compilation unit exports this name, preventing symbol conflicts.
#[unsafe(export_name = "DriverEntry")] // WDF expects a symbol with the name DriverEntry
pub unsafe extern "system" fn driver_entry(
    driver: &mut DRIVER_OBJECT,
    _registry_path: PCUNICODE_STRING,
) -> NTSTATUS {
    unsafe {
        (*driver).DriverUnload = Some(zero_unload);
        (*driver).MajorFunction[IRP_MJ_CREATE as usize] = Some(zero_create_close);
        (*driver).MajorFunction[IRP_MJ_CLOSE as usize] = Some(zero_create_close);
        (*driver).MajorFunction[IRP_MJ_READ as usize] = Some(zero_read);
        (*driver).MajorFunction[IRP_MJ_WRITE as usize] = Some(zero_write);
        (*driver).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = Some(zero_device_control);
        
        let mut status;
        let mut device_object = PDEVICE_OBJECT::default();
        let mut symlink_created = false;

        loop {
            status = IoCreateDevice(driver, 0, &DEVICE_NAME as *const _ as *mut _, FILE_DEVICE_UNKNOWN, 0, false.into(), &mut device_object);
            if !NT_SUCCESS(status) {
                DbgPrint(b"%sfailed to create device (0x%08X)\n" as *const _ as *const i8, DRIVER_PREFIX.as_ptr(), status);
                break;
            }
            (*device_object).Flags |= DO_DIRECT_IO;

            status = IoCreateSymbolicLink(& DEVICE_SYMLINK as *const _ as *mut _, & DEVICE_NAME as *const _ as *mut _);
            if !NT_SUCCESS(status) {
                DbgPrint("%sfailed to create symbolic link (0x%08X)\n" as *const _ as *const i8, DRIVER_PREFIX.as_ptr(), status);
                IoDeleteDevice(device_object);
                break;
            }
            symlink_created = true;
            break;
        }
    
        if !NT_SUCCESS(status) {
            if symlink_created {
                let _ = IoDeleteSymbolicLink(&DEVICE_SYMLINK as *const _ as *mut _);
            }
            if !device_object.is_null() {
                IoDeleteDevice(device_object);
            }
        }

        status
    }
}

unsafe extern "C" fn zero_unload(driver: *mut DRIVER_OBJECT) {
    println!("Booster: Driver unload");

    unsafe {
        let _ = IoDeleteSymbolicLink(&DEVICE_SYMLINK as *const _ as *mut _);
        IoDeleteDevice((*driver).DeviceObject);
    }
}

unsafe fn complete_irp(irp: *mut wdk_sys::IRP, status: NTSTATUS, information: usize) -> NTSTATUS {
    unsafe {
        (*irp).IoStatus.__bindgen_anon_1.Status = status;
        (*irp).IoStatus.Information = information as u64;
        IofCompleteRequest(irp, IO_NO_INCREMENT as i8);
        status
    }
}

unsafe extern "C" fn zero_create_close(
    _device: *mut wdk_sys::DEVICE_OBJECT,
    irp: *mut wdk_sys::IRP,
) -> NTSTATUS {
    unsafe {
        complete_irp(irp, STATUS_SUCCESS, 0)
    }
}

unsafe extern "C" fn zero_read(
    _device: *mut wdk_sys::DEVICE_OBJECT,
    irp: *mut wdk_sys::IRP,
) -> NTSTATUS {
    unsafe {
        let stack = IoGetCurrentIrpStackLocation(irp);
        let len = (*stack).Parameters.Read.Length;
        if len == 0 {
            return complete_irp(irp, STATUS_INVALID_BUFFER_SIZE, 0);
        }
        complete_irp(irp, STATUS_SUCCESS, 0)
    }
}

unsafe extern "C" fn zero_write(
    _device: *mut wdk_sys::DEVICE_OBJECT,
    _irp: *mut wdk_sys::IRP,
) -> NTSTATUS {
    todo!()
}

unsafe extern "C" fn zero_device_control(
    _device: *mut wdk_sys::DEVICE_OBJECT,
    _irp: *mut wdk_sys::IRP,
) -> NTSTATUS {
    todo!()
}
