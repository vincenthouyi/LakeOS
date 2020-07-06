use core::fmt::{Write, Arguments, Result};

struct DebugPrinter {}

impl Write for DebugPrinter {
    fn write_str(&mut self, s: &str) -> Result {
        for c in s.chars() {
            let msg_len = 1;
            let msg_info = crate::MsgInfo::new(crate::SyscallOp::DebugPrint, msg_len);

            unsafe {
                let mut args = [0, c as usize,0,0,0,0];
                crate::syscall(msg_info, &mut args).unwrap();
            }
        }
        Ok(())
    }
}

pub fn _print(args: Arguments) {
    let mut debug_printer = DebugPrinter{};
    debug_printer.write_fmt(args).unwrap();
}

/// Like `println!`, but for kernel-space.
pub macro debug_println {
    () => (print!("\n")),
    ($fmt:expr) => (debug_print!(concat!($fmt, "\n"))),
    ($fmt:expr, $($arg:tt)*) => (debug_print!(concat!($fmt, "\n"), $($arg)*))
}

/// Like `print!`, but for kernel-space.
pub macro debug_print($($arg:tt)*) {
    _print(format_args!($($arg)*))
}
