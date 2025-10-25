use std::mem::size_of;
use core::ffi::c_void;

use windows::Wdk::Foundation::OBJECT_ATTRIBUTES;
use windows::Win32::Devices::Beep::{DD_BEEP_DEVICE_NAME_U, IOCTL_BEEP_SET};
use windows::Win32::Foundation::{CloseHandle, GENERIC_WRITE, HANDLE, OBJECT_ATTRIBUTE_FLAGS, UNICODE_STRING};
use windows::Win32::System::Threading::Sleep;
use windows::Win32::System::WindowsProgramming::RtlInitUnicodeString;
use windows::Win32::System::IO::{DeviceIoControl, IO_STATUS_BLOCK};
use windows::Wdk::Storage::FileSystem::NtOpenFile;

#[repr(C)]
#[derive(Clone, Copy)]
struct BeepSetParameters {
    frequency: u32,
    duration: u32,
}

#[derive(Debug, Default, Clone)]
struct Handle(pub HANDLE);

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_invalid() {
                let _ = CloseHandle(self.0);
            }
        }
    }
}

fn main() {
    // simple CLI parsing
    let args: Vec<String> = std::env::args().collect();
    println!("beep [<frequency> <duration_in_msec>]");
    let mut freq: u32 = 800;
    let mut duration: u32 = 1000;
    if args.len() > 2 {
        if let Ok(f) = args[1].parse::<u32>() { freq = f; }
        if let Ok(d) = args[2].parse::<u32>() { duration = d; }
    }

    unsafe {
        let mut handle = Handle::default();
        let mut name = UNICODE_STRING::default();
        RtlInitUnicodeString(&mut name, DD_BEEP_DEVICE_NAME_U);
        let attr = OBJECT_ATTRIBUTES {
            Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: HANDLE(std::ptr::null_mut()),
            ObjectName: &name,
            Attributes: OBJECT_ATTRIBUTE_FLAGS::default(),
            SecurityDescriptor: std::ptr::null_mut(),
            SecurityQualityOfService: std::ptr::null_mut(),
        };
        let mut io_status = IO_STATUS_BLOCK::default();
        let status = NtOpenFile(
            &mut handle.0,
            GENERIC_WRITE.0,
            &attr,
            &mut io_status,
            0,
            0);

        if status.is_ok() {
            let params = BeepSetParameters { frequency: freq, duration };
            let mut bytes_returned: u32 = 0;

            println!("Playing freq: {freq}, duration: {duration}");

            DeviceIoControl(
                handle.0,
                IOCTL_BEEP_SET,
                Some(&params as *const _ as *const c_void),
                size_of::<BeepSetParameters>() as u32,
                None,
                0,
                Some(&mut bytes_returned as *mut u32),
                None,
            ).expect("DeviceIoControl failed");

            Sleep(duration);
        }
        else {
            eprintln!("Failed in NtOpenFile (status=0x{:X})", status.0);
        }
    }
}
