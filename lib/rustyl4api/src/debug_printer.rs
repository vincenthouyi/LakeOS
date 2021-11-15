use core::fmt::{Arguments, Result, Write};
use spin::Mutex;

use crate::syscall::{syscall, MsgInfo, SyscallOp};
use crate::thread::{cpu_id, thread_id};
use log::{Log, Metadata, Record};

pub struct KernelConsole{}
pub struct DebugPrinter(Mutex<KernelConsole>);
pub static DEBUG_PRINTER: DebugPrinter = DebugPrinter::new();

impl DebugPrinter {
    pub const fn new() -> Self {
        Self(Mutex::new(KernelConsole{}))
    }
}

impl Write for KernelConsole {
    fn write_str(&mut self, s: &str) -> Result {
        for c in s.chars() {
            let msg_len = 1;
            let msg_info = MsgInfo::new(SyscallOp::DebugPrint, msg_len);

            let mut args = [0, c as usize, 0, 0, 0, 0];
            syscall(msg_info, &mut args).unwrap();
        }
        Ok(())
    }
}

impl Log for DebugPrinter {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            self.0
                .lock()
                .write_fmt(format_args!("[Thread-{}:{:x}-{}] {}\n", cpu_id(), thread_id(), record.level(), record.args()))
                .expect("fail to print to kernel console!");
        }
    }

    fn flush(&self) {}
}

pub fn _print(args: Arguments) {
    DEBUG_PRINTER.0
        .lock()
        .write_fmt(args).unwrap();
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
