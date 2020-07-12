pub use crate::plat::uart::console_print;
use crate::arch::affinity;

/// Like `println!`, but for kernel-space.
pub macro kprintln {
    () => (kprint!("[Kernel:{}]\n", affinity())),
    ($fmt:expr) => (kprint!(concat!("[Kernel:{}] ", $fmt, "\n"), affinity())),
    ($fmt:expr, $($arg:tt)*) => (kprint!(concat!("[Kernel:{}] ",$fmt, "\n"), affinity(),$($arg)*))
}

/// Like `print!`, but for kernel-space.
pub macro kprint($($arg:tt)*) {
    console_print(format_args!($($arg)*))
}
