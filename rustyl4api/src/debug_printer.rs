use core::fmt::{Write, Arguments, Result};

use crate::syscall::{syscall, MsgInfo, SyscallOp};

struct DebugPrinter {}

impl Write for DebugPrinter {
    fn write_str(&mut self, s: &str) -> Result {
        for c in s.chars() {
            let msg_len = 1;
            let msg_info = MsgInfo::new(SyscallOp::DebugPrint, msg_len);

            let mut args = [0, c as usize,0,0,0,0];
            syscall(msg_info, &mut args).unwrap();
        }
        Ok(())
    }
}

pub fn _print(args: Arguments) {
    let mut debug_printer = DebugPrinter{};
    debug_printer.write_fmt(args).unwrap();
}

/// Like `println!`, but for kernel-space.
pub macro kprintln {
    () => (print!("\n")),
    ($fmt:expr) => (kprint!(concat!($fmt, "\n"))),
    ($fmt:expr, $($arg:tt)*) => (kprint!(concat!($fmt, "\n"), $($arg)*))
}

/// Like `print!`, but for kernel-space.
pub macro kprint($($arg:tt)*) {
    _print(format_args!($($arg)*))
}
