pub use crate::plat::uart::console_print;

/// Like `println!`, but for kernel-space.
pub macro kprintln {
    () => (kprint!("[Kernel:{}]\n", cpuid())),
    ($fmt:expr) => (kprint!(concat!("[Kernel:{}] ", $fmt, "\n"), cpuid())),
    ($fmt:expr, $($arg:tt)*) => (kprint!(concat!("[Kernel:{}] ",$fmt, "\n"), cpuid(),$($arg)*))
}

/// Like `print!`, but for kernel-space.
pub macro kprint($($arg:tt)*) {
    console_print(format_args!($($arg)*))
}
