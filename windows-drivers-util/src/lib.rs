#![no_std]

use wdk_sys::{
   PIO_STACK_LOCATION, PIRP, MDL, MDL_MAPPED_TO_SYSTEM_VA, MDL_SOURCE_IS_NONPAGED_POOL,
};

/// Macro to print debug messages to the kernel debugger.
/// Courtesy of mhandb: https://github.com/microsoft/windows-drivers-rs/discussions/17
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! kd_print {
    ($msg: expr) => {
        #[allow(unused_unsafe)]
        unsafe {
            DbgPrint(concat!($msg, "\0").as_ptr() as *mut i8)
        }
    };

    ($format: expr, $($arg:tt)*) => {
        #[allow(unused_unsafe)]
        unsafe {
             DbgPrint(concat!($format, "\0").as_ptr() as *mut i8, $($arg)*)
        }
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! kd_print {
    ($msg: expr) => {};
    ($format: expr, $($arg:tt)*) => {};
}

/// This routine is invoked to return a pointer to the current stack location
/// in an I/O Request Packet (IRP).
///
/// # Arguments
/// * `irp` - Pointer to the I/O Request Packet.
/// 
/// # Returns
/// The function value is a pointer to the current stack location in the
/// packet.
#[allow(non_snake_case)]
pub fn IoGetCurrentIrpStackLocation(irp: PIRP) -> PIO_STACK_LOCATION {
    unsafe {
        let Some(irp) = irp.as_ref() else {
            panic!("irp pointer is null");
        };

        assert!(irp.CurrentLocation <= irp.StackCount + 1);
        irp.Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation
    }
}

/// This routine returns the mapped address of an MDL. If the
/// Mdl is not already mapped or a system address, it is mapped.
/// 
/// # Arguments
/// * `Mdl` - Pointer to the MDL to map.
/// * `Priority` - Supplies an indication as to how important it is that this
///                request succeed under low available PTE conditions.
/// 
/// # Returns
/// Returns the base address where the pages are mapped.  The base address
/// has the same offset as the virtual address in the MDL.
/// Unlike MmGetSystemAddressForMdl, Safe guarantees that it will always
/// return NULL on failure instead of bugchecking the system.
/// This routine is not usable by WDM 1.0 drivers as 1.0 did not include
/// MmMapLockedPagesSpecifyCache.  The solution for WDM 1.0 drivers is to
/// provide synchronization and set/reset the MDL_MAPPING_CAN_FAIL bit.
#[allow(non_snake_case)]
pub fn MmGetSystemAddressForMdlSafe(Mdl: *mut MDL, Priority: u32) -> *mut core::ffi::c_void {
    unsafe {
        if (*Mdl).MdlFlags & ((MDL_MAPPED_TO_SYSTEM_VA | MDL_SOURCE_IS_NONPAGED_POOL) as i16) != 0 {
            (*Mdl).MappedSystemVa
        } else {
            wdk_sys::ntddk::MmMapLockedPagesSpecifyCache(
                Mdl,
                wdk_sys::_MODE::KernelMode as i8,
                wdk_sys::_MEMORY_CACHING_TYPE::MmCached,
                core::ptr::null_mut(),
                false.into(),
                Priority,
            )
        }
    }
}
