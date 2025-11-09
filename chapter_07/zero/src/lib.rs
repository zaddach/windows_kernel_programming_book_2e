#![no_std]

use wdk::println;
use wdk_strings::u;
use wdk_sys::{
    _MM_PAGE_PRIORITY::NormalPagePriority, DO_DIRECT_IO, DRIVER_OBJECT, FILE_DEVICE_UNKNOWN, IO_NO_INCREMENT, IRP_MJ_CLOSE, IRP_MJ_CREATE, IRP_MJ_DEVICE_CONTROL, IRP_MJ_READ, IRP_MJ_WRITE, NT_SUCCESS, NTSTATUS, PCUNICODE_STRING, PDEVICE_OBJECT, STATUS_INSUFFICIENT_RESOURCES, STATUS_INVALID_BUFFER_SIZE, STATUS_SUCCESS, STATUS_BUFFER_TOO_SMALL, STATUS_INVALID_PARAMETER, STATUS_INVALID_DEVICE_REQUEST, UNICODE_STRING, ntddk::{
        DbgPrint, IoCreateDevice, IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink, IofCompleteRequest
    }
};
use windows_drivers_util::{IoGetCurrentIrpStackLocation, MmGetSystemAddressForMdlSafe};

#[cfg(not(test))]
extern crate wdk_panic;

#[cfg(not(test))]
use wdk_alloc::WdkAllocator;
use zero_common::{IOCTL_ZERO_CLEAR_STATS, IOCTL_ZERO_GET_STATS, ZeroStats};

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WdkAllocator = WdkAllocator;

const DEVICE_NAME: UNICODE_STRING = u!(r"\Device\Zero");
const DEVICE_SYMLINK: UNICODE_STRING = u!(r"\??\Zero");
const DRIVER_PREFIX: &[u8] = b"Zero: ";

static mut TOTAL_READ: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
static mut TOTAL_WRITTEN: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

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

#[allow(static_mut_refs)]
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

        let buffer = MmGetSystemAddressForMdlSafe((*irp).MdlAddress, NormalPagePriority as u32);
        if buffer.is_null() {
            return complete_irp(irp, STATUS_INSUFFICIENT_RESOURCES, 0);
        }
        core::slice::from_raw_parts_mut(buffer as *mut u8, len as usize).fill(0);
        let _ = TOTAL_READ.fetch_add(len as u64, core::sync::atomic::Ordering::Relaxed);
        complete_irp(irp, STATUS_SUCCESS, len as usize)
    }
}

#[allow(static_mut_refs)]
unsafe extern "C" fn zero_write(
    _device: *mut wdk_sys::DEVICE_OBJECT,
    irp: *mut wdk_sys::IRP,
) -> NTSTATUS {
    unsafe {
        let stack = IoGetCurrentIrpStackLocation(irp);
        let len = (*stack).Parameters.Write.Length;
        let _ = TOTAL_WRITTEN.fetch_add(len as u64, core::sync::atomic::Ordering::Relaxed);
        complete_irp(irp, STATUS_SUCCESS, len as usize)
    }
}

#[allow(static_mut_refs)]
unsafe extern "C" fn zero_device_control(
    _device: *mut wdk_sys::DEVICE_OBJECT,
    irp: *mut wdk_sys::IRP,
) -> NTSTATUS {
    unsafe {
        let irp_sp = IoGetCurrentIrpStackLocation(irp);
        let dic = &(*irp_sp).Parameters.DeviceIoControl;
        let mut status = STATUS_INVALID_DEVICE_REQUEST;
        let mut len: usize = 0;

        match dic.IoControlCode {
            IOCTL_ZERO_GET_STATS => {
                if dic.OutputBufferLength < core::mem::size_of::<ZeroStats>() as u32 {
                    status = STATUS_BUFFER_TOO_SMALL;
                } else {
                    let stats = (*irp).AssociatedIrp.SystemBuffer as *mut ZeroStats;
                    if stats.is_null() {
                        status = STATUS_INVALID_PARAMETER;
                    } else {
                        (*stats).total_read = TOTAL_READ.load(core::sync::atomic::Ordering::Relaxed);
                        (*stats).total_written = TOTAL_WRITTEN.load(core::sync::atomic::Ordering::Relaxed);
                        len = core::mem::size_of::<ZeroStats>();
                        status = STATUS_SUCCESS;
                    }
                }
            }
            IOCTL_ZERO_CLEAR_STATS => {
                TOTAL_READ.store(0, core::sync::atomic::Ordering::Relaxed);
                TOTAL_WRITTEN.store(0, core::sync::atomic::Ordering::Relaxed);
                status = STATUS_SUCCESS;
            }
            _ => {
                // status is already set to STATUS_INVALID_DEVICE_REQUEST
            }
        }

        complete_irp(irp, status, len)
    }
}
