use std::ffi::c_int;

#[repr(C)]
pub struct ThreadData {
    pub thread_id: u32,
    pub priority: c_int,
}
