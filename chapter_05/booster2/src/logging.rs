#[allow(dead_code)]
pub enum LogLevel {
    Error = 0,
    Warning = 1,
    Info = 2,
    Debug = 3,
    Verbose = 4,
}

#[macro_export]
macro_rules! log {
    ($level:expr, $format:literal) => {
        #[cfg(debug_assertions)]
        #[allow(unused_unsafe)]
        unsafe {
            wdk_sys::ntddk::DbgPrintEx(wdk_sys::_DPFLTR_TYPE::DPFLTR_IHVDRIVER_ID as u32, $level as u32, const_str::concat_bytes!(b"Booster2: ", $format, b"\0") as *const _ as *const i8);
        }

        #[cfg(not(debug_assertions))]
        { /* no-op in release builds */ }
    };

    ($level:expr, $format:literal, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        #[allow(unused_unsafe)]
        unsafe {
            wdk_sys::ntddk::DbgPrintEx(wdk_sys::_DPFLTR_TYPE::DPFLTR_IHVDRIVER_ID as u32, $level as u32, const_str::concat_bytes!(b"Booster2: ", $format, b"\0") as *const _ as *const i8, $($arg)*);
        }

        #[cfg(not(debug_assertions))]
        { /* no-op in release builds */ }
    };
}

#[macro_export]
macro_rules! log_info {
    ($format:literal) => {
        log!(LogLevel::Info as u32, $format);
    };

    ($format:literal, $($arg:tt)*) => {
        log!(LogLevel::Info as u32, $format, $($arg)*);
    };
}

#[macro_export]
macro_rules! log_error {
    ($format:literal) => {
        log!(LogLevel::Error as u32, $format);
    };

    ($format:literal, $($arg:tt)*) => {
        log!(LogLevel::Error as u32, $format, $($arg)*);
    };
}
