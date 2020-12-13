use spin::Mutex;
use core::fmt::{Write, Arguments, Result};

use crate::syscall::{syscall, MsgInfo, SyscallOp};

pub struct DebugPrinter {}
pub static DEBUG_PRINTER: Mutex::<DebugPrinter> = Mutex::new(DebugPrinter{});

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
    let mut debug_printer = DEBUG_PRINTER.lock();
    debug_printer.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::debug_printer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::kprint!("\n"));
    ($($arg:tt)*) => ($crate::kprint!("[Thread-{}:{:x}] {}\n", $crate::thread::cpu_id(), $crate::thread::thread_id(), format_args!($($arg)*)));
}