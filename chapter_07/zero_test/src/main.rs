use windows::Win32::Foundation::{CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_MODE, OPEN_EXISTING, WriteFile,
    ReadFile,
};
use windows::Win32::System::IO::DeviceIoControl;
use windows_strings::{PCWSTR, w};
use zero_common::{IOCTL_ZERO_GET_STATS, ZeroStats};

const DEVICE_PATH: PCWSTR = w!(r"\\.\Zero");

/// A wrapper around HANDLE that ensures the handle is closed when dropped.
struct Handle(HANDLE);

impl Drop for Handle {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}

fn main() {
    unsafe {
        let device = Handle(
            CreateFileW(
                DEVICE_PATH,
                GENERIC_READ.0 | GENERIC_WRITE.0,
                FILE_SHARE_MODE::default(),
                None,
                OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES::default(),
                None,
            )
            .expect("Failed to open device"),
        );

        println!("Test read");
        let mut buffer = [0u8; 64];
        for (idx, byte) in buffer.iter_mut().enumerate() {
            *byte = (idx + 1) as u8;
        }

        let mut bytes_read = 0;
        ReadFile(
            device.0,
            Some(&mut buffer),
            Some(&mut bytes_read),
            None,
        )
        .expect("failed to read");

        if bytes_read as usize != buffer.len() {
            panic!(
                "Expected to read {} bytes, but read {} bytes",
                buffer.len(),
                bytes_read
            );
        }

        for byte in buffer.iter() {
            if *byte != 0 {
                panic!("Expected all zeroes, but found non-zero byte: {}", byte);
            }
        }

        println!("Test write");
        let buffer = [0x55u8; 1024];
        let mut bytes_written = 0;
        WriteFile(
            device.0,
            Some(&buffer),
            Some(&mut bytes_written),
            None,
        )
        .expect("failed to write");

        if bytes_written as usize != buffer.len() {
            panic!(
                "Expected to write {} bytes, but wrote {} bytes",
                buffer.len(),
                bytes_written
            );
        }

        let mut stats = ZeroStats::default();
        let mut bytes_read = 0;
        DeviceIoControl(
            device.0,
            IOCTL_ZERO_GET_STATS,
            None,
            0,
            Some(&mut stats as *mut _ as *mut core::ffi::c_void),
            core::mem::size_of::<ZeroStats>() as u32,
            Some(&mut bytes_read),
            None,
        )
        .expect("failed in DeviceIoControl");

        debug_assert_eq!(bytes_read as usize, core::mem::size_of::<ZeroStats>());
        println!("Total Read: {}, Total Write: {}", stats.total_read, stats.total_written);
    }
}
