#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => (crate::uart::console_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::kprint!("\n"));
    ($($arg:tt)*) => ($crate::kprint!("[Bootloader] {}\n", format_args!($($arg)*)));
}
