use booster_common::ThreadData;
use core::slice;
use std::env;
use windows::Win32::Foundation::{CloseHandle, GENERIC_WRITE, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_MODE, OPEN_EXISTING, WriteFile,
};
use windows_strings::{PCWSTR, w};

const DEVICE_PATH: PCWSTR = w!(r"\\.\Booster");

/// A wrapper around HANDLE that ensures the handle is closed when dropped.
struct Handle(HANDLE);

impl Drop for Handle {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: boost <tid> <priority>");
        std::process::exit(1);
    }

    let Ok(tid) = args[1].parse::<u32>() else {
        println!("Invalid thread ID");
        std::process::exit(1);
    };

    let priority = args[2].parse::<i32>().unwrap_or_else(|_| {
        eprintln!("Invalid priority value");
        std::process::exit(1);
    });

    unsafe {
        let device = Handle(
            CreateFileW(
                DEVICE_PATH,
                GENERIC_WRITE.0,
                FILE_SHARE_MODE::default(),
                None,
                OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES::default(),
                None,
            )
            .expect("Failed to open device"),
        );

        let data = ThreadData {
            thread_id: tid,
            priority: priority,
        };

        let mut bytes_written = 0;
        WriteFile(
            device.0,
            Some(slice::from_raw_parts(
                &data as *const _ as *const u8,
                std::mem::size_of::<ThreadData>(),
            )),
            Some(&mut bytes_written),
            None,
        )
        .expect("Priority change failed");

        println!("Priority change succeeded!");
    }
}
