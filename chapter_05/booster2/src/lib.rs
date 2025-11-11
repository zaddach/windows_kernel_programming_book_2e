#![no_std]

mod logging;

use core::ffi::c_void;

use booster_common::ThreadData;
use wdk_strings::u;
use wdk_sys::{
    DRIVER_OBJECT, FILE_DEVICE_UNKNOWN, HANDLE, IO_NO_INCREMENT, NT_SUCCESS, NTSTATUS,
    PCUNICODE_STRING, PDEVICE_OBJECT, PETHREAD, STATUS_BUFFER_TOO_SMALL, STATUS_INVALID_PARAMETER,
    STATUS_SUCCESS, UNICODE_STRING,
    ntddk::{
        IoCreateDevice, IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink,
        IofCompleteRequest, KeSetPriorityThread, ObfDereferenceObject, PsLookupThreadByThreadId,
    },
};
use windows_drivers_util::IoGetCurrentIrpStackLocation;

#[cfg(not(test))]
extern crate wdk_panic;

#[cfg(not(test))]
use wdk_alloc::WdkAllocator;

use crate::logging::LogLevel;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WdkAllocator = WdkAllocator;

const DEVICE_NAME: UNICODE_STRING = u!(r"\Device\Booster");
const DEVICE_SYMLINK: UNICODE_STRING = u!(r"\??\Booster");

// SAFETY: "DriverEntry" is the required symbol name for Windows driver entry points.
// No other function in this compilation unit exports this name, preventing symbol conflicts.
#[unsafe(export_name = "DriverEntry")] // WDF expects a symbol with the name DriverEntry
pub unsafe extern "system" fn driver_entry(
    driver: &mut DRIVER_OBJECT,
    registry_path: PCUNICODE_STRING,
) -> NTSTATUS {
    unsafe {
        log!(
            LogLevel::Info as u32,
            b"DriverEntry started. Registry Path: %wZ\n",
            registry_path
        );
        let Some(registry_path) = registry_path.as_ref() else {
            log_error!(b"DriverEntry failed to get registry path");
            return STATUS_INVALID_PARAMETER;
        };

        log!(LogLevel::Info as u32, b"Registry Path: %wZ", registry_path);
    }
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
            &mut device_object,
        );

        if !NT_SUCCESS(status) {
            log_error!("Failed to create device object (0x{:08X})\n", status);
            return status;
        }

        debug_assert!(!device_object.is_null());

        let status = IoCreateSymbolicLink(
            &DEVICE_SYMLINK as *const _ as *mut _,
            &DEVICE_NAME as *const _ as *mut _,
        );
        if !NT_SUCCESS(status) {
            log_error!(b"Failed to create symbolic link (0x{:08X})\n", status);

            IoDeleteDevice(device_object); // Important
            return status;
        }
    }

    STATUS_SUCCESS
}

unsafe extern "C" fn booster_unload(driver: *mut DRIVER_OBJECT) {
    log_info!("Booster2 unload called\n");

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
    log!(LogLevel::Verbose as u32, b"Booster2: create/close called\n");
    unsafe {
        if let Some(irp) = irp.as_mut() {
            irp.IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            irp.IoStatus.Information = 0;
            IofCompleteRequest(irp, wdk_sys::IO_NO_INCREMENT as i8);
        } else {
            log_error!(b"Create/Close request received with null IRP\n");
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
                        status = PsLookupThreadByThreadId(data.thread_id as HANDLE, &mut thread);
                        if !NT_SUCCESS(status) {
                            log_error!(
                                "Failed to locate thread %u (0x%X)\n",
                                data.thread_id,
                                status
                            );
                            break;
                        }

                        let old_priority = KeSetPriorityThread(thread, data.priority);
                        log_info!(
                            b"Priority for thread %u changed from %d to %d\n",
                            data.thread_id,
                            old_priority,
                            data.priority
                        );

                        ObfDereferenceObject(thread as *mut c_void);
                        information = core::mem::size_of::<ThreadData>() as u64;
                    } else {
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
