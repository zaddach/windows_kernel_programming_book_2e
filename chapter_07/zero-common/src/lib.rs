#[cfg(feature = "kernel")]
pub use wdk_sys::{
    METHOD_BUFFERED, METHOD_NEITHER, FILE_ANY_ACCESS,
};
#[cfg(feature = "user")]
pub use windows::Win32::System::Ioctl::{
    METHOD_BUFFERED, METHOD_NEITHER, FILE_ANY_ACCESS,
};
use windows_driver_common_util::ctl_code;

pub const DEVICE_ZERO: u32 = 0x8022;

pub const IOCTL_ZERO_GET_STATS: u32 = ctl_code!(DEVICE_ZERO, 0x800, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const IOCTL_ZERO_CLEAR_STATS: u32 = ctl_code!(DEVICE_ZERO, 0x801, METHOD_NEITHER, FILE_ANY_ACCESS);

#[derive(Default)]
pub struct ZeroStats {
    pub total_read: u64,
    pub total_written: u64,
}
