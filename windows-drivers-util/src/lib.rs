#![no_std]

use wdk_sys::{
   PIO_STACK_LOCATION, PIRP,
};

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
