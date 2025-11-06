#[macro_export]
macro_rules! ctl_code {
    ($device_type:expr, $function:expr, $method:expr, $access:expr) => {
        ((($device_type & 0xffff) as u32) << 16)
            | ((($access & 0x3) as u32) << 14)
            | ((($function & 0xfff) as u32) << 2)
            | (($method & 0x3) as u32)
    };
}