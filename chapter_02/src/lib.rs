#![no_std]

use core::ffi::c_void;

use wdk::println;
use wdk_sys::{
   ntddk::{ExAllocatePool2, ExFreePool, RtlCopyUnicodeString, RtlGetVersion}, DRIVER_OBJECT, NTSTATUS, PCUNICODE_STRING, PDRIVER_OBJECT, RTL_OSVERSIONINFOW, SIZE_T, STATUS_INSUFFICIENT_RESOURCES, STATUS_INVALID_PARAMETER, STATUS_SUCCESS, UNICODE_STRING, _POOL_TYPE::PagedPool
};

#[cfg(not(test))]
extern crate wdk_panic;

#[cfg(not(test))]
use wdk_alloc::WdkAllocator;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WdkAllocator = WdkAllocator;

const DRIVER_TAG: u32 = u32::from_ne_bytes(*b"dcba");

static mut REGISTRY_PATH: UNICODE_STRING = UNICODE_STRING {
   Length: 0,
   MaximumLength: 0,
   Buffer: core::ptr::null_mut(),
};

unsafe extern "C" fn sample_unload(_driver: PDRIVER_OBJECT) {
	unsafe {
        ExFreePool(REGISTRY_PATH.Buffer as *mut c_void);
	    println!("Sample driver Unload called");
    }
}

// SAFETY: "DriverEntry" is the required symbol name for Windows driver entry points.
// No other function in this compilation unit exports this name, preventing symbol conflicts.
#[unsafe(export_name = "DriverEntry")] // WDF expects a symbol with the name DriverEntrypub unsafe extern "system" fn 
#[allow(static_mut_refs)]
fn driver_entry(
   driver: &mut DRIVER_OBJECT,
   registry_path: PCUNICODE_STRING,
) -> NTSTATUS {
    unsafe {
        let Some(registry_path) = registry_path.as_ref() else {
            return STATUS_INVALID_PARAMETER;
        };

        REGISTRY_PATH.Buffer = ExAllocatePool2(PagedPool as u64, registry_path.Length as SIZE_T, DRIVER_TAG) as *mut u16;
        if REGISTRY_PATH.Buffer.is_null() {
            println!("Failed to allocate memory");
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        REGISTRY_PATH.MaximumLength = registry_path.Length;
        RtlCopyUnicodeString(&mut REGISTRY_PATH, registry_path);
        println!("original registry path: {:?}", registry_path);
        println!("Copied registry path: {:?}", &REGISTRY_PATH);

        driver.DriverUnload = Some(sample_unload);
        
        let mut info = RTL_OSVERSIONINFOW::default();
        let _ = RtlGetVersion(&mut info);
        println!("Windows version: {}.{}.{}", info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber);

        println!("Sample driver initialized successfully");
    }
    
    STATUS_SUCCESS
}
