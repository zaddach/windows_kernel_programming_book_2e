#![no_std]

use core::ffi::c_void;

use booster_common::ThreadData;
use tracelogging::{define_provider, write_event};
use wdk::println;
use wdk_strings::u;
use wdk_sys::{
    DRIVER_OBJECT, FILE_DEVICE_UNKNOWN, HANDLE, IO_NO_INCREMENT, IRP_MJ_CREATE, NT_SUCCESS, NTSTATUS, PCUNICODE_STRING, PDEVICE_OBJECT, PETHREAD, STATUS_BUFFER_TOO_SMALL, STATUS_INVALID_PARAMETER, STATUS_SUCCESS, UNICODE_STRING, ntddk::{
        IoCreateDevice, IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink,
        IofCompleteRequest, KeSetPriorityThread, ObfDereferenceObject, PsLookupThreadByThreadId,
    }
};
use windows_drivers_util::IoGetCurrentIrpStackLocation;

#[cfg(not(test))]
extern crate wdk_panic;

#[cfg(not(test))]
use wdk_alloc::WdkAllocator;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WdkAllocator = WdkAllocator;

const DEVICE_NAME: UNICODE_STRING = u!(r"\Device\Booster");
const DEVICE_SYMLINK: UNICODE_STRING = u!(r"\??\Booster");

define_provider!(
    BOOSTER_PROVIDER,
    "Booster",
    id("b2723ad5-1678-446d-a577-8599d3e85ecb"),
);

// SAFETY: "DriverEntry" is the required symbol name for Windows driver entry points.
// No other function in this compilation unit exports this name, preventing symbol conflicts.
#[unsafe(export_name = "DriverEntry")] // WDF expects a symbol with the name DriverEntry
pub unsafe extern "system" fn driver_entry(
    driver: &mut DRIVER_OBJECT,
    registry_path: PCUNICODE_STRING,
) -> NTSTATUS {
    unsafe {
        BOOSTER_PROVIDER.register();
        let Some(registry_path) = registry_path.as_ref() else {
            write_event!(
                BOOSTER_PROVIDER,
                "DriverEntry failed to get registry path",
                level(tracelogging::Level::Error),
            );
            return STATUS_INVALID_PARAMETER;
        };

        write_event!(
            BOOSTER_PROVIDER,
            "DriverEntry started",
            level(tracelogging::Level::Informational),
            cstr8("DriverName", "Booster Driver"),
            str16("RegistryPath", core::slice::from_raw_parts(registry_path.Buffer, registry_path.Length as usize / 2)),
        );
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
            println!("Failed to create device object (0x{:08X})", status);
            return status;
        }

        debug_assert!(!device_object.is_null());

        let status = IoCreateSymbolicLink(
            &DEVICE_SYMLINK as *const _ as *mut _,
            &DEVICE_NAME as *const _ as *mut _,
        );
        if !NT_SUCCESS(status) {
            write_event!(
                BOOSTER_PROVIDER,
                "Error",
                level(tracelogging::Level::Error),
                cstr8("Message", "Symbolic link creation failed"),
                u32("Status", &(status as u32)),
            );

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
        write_event!(
            BOOSTER_PROVIDER,
            "Unload",
            level(tracelogging::Level::Informational),
            cstr8("Message", "Driver unloading"),   
        );
    }
}

unsafe extern "C" fn booster_create_close(
    _device: *mut wdk_sys::DEVICE_OBJECT,
    irp: *mut wdk_sys::IRP,
) -> NTSTATUS {
    unsafe {
        if let Some(irp) = irp.as_mut() {
            write_event!(
                BOOSTER_PROVIDER,
                "Create/Close",
                level(tracelogging::Level::Informational),
                cstr8("Operation", if IoGetCurrentIrpStackLocation(irp).as_ref().unwrap().MajorFunction as u32 == IRP_MJ_CREATE {"Create"} else {"Close"}),
            );
            irp.IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            irp.IoStatus.Information = 0;
            IofCompleteRequest(irp, wdk_sys::IO_NO_INCREMENT as i8);
        }
        else {
            write_event!(
                BOOSTER_PROVIDER,
                "Create/Close failed",
                level(tracelogging::Level::Error),
            );
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
                            break;
                        }

                        let old_priority = KeSetPriorityThread(thread, data.priority);
                        write_event!(
                            BOOSTER_PROVIDER,
                            "Boosting",
                            level(tracelogging::Level::Informational),
                            u64("ThreadId", &(data.thread_id as u64)),
                            u32("OldPriority", &(old_priority as u32)),
                            u32("NewPriority", &(data.priority as u32)),
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
