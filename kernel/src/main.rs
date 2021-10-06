#![allow(incomplete_features)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(maybe_uninit_ref)]
#![feature(const_generics)]
#![feature(llvm_asm)]
#![allow(non_snake_case)]
#![no_std]
#![no_main]
#![allow(non_upper_case_globals)]
#![feature(naked_functions)]
#![feature(global_asm)]

#[macro_use]
extern crate static_assertions;
#[macro_use]
extern crate num_derive;
#[macro_use]
mod macros;
#[macro_use]
mod console;
#[macro_use]
mod arch;
mod cspace;
mod interrupt;
mod objects;
mod plat;
mod scheduler;
mod syscall;
mod utils;

pub use scheduler::SCHEDULER;

pub const TICK: u32 = 1000;
pub const TIME_SLICE: isize = 1000;
pub const NCPU: usize = 4;

mod prelude {
    pub use crate::console::{kprint, kprintln};
    pub use core::convert::TryFrom;
    pub use rustyl4api as sysapi;
    pub use sysapi::error::{SysError, SysResult};

    pub const PHYS_BASE: usize = 0x80000;
    pub const KERNEL_BASE: usize = 0xffff0000_00080000;
    pub const KERNEL_OFFSET: usize = KERNEL_BASE - PHYS_BASE;
    pub const PHYS_IO_BASE: usize = 0x3f000000;
    pub const IO_BASE: usize = PHYS_IO_BASE + KERNEL_OFFSET;
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use crate::prelude::*;
    kprintln!("Panic! {:?}", info);
    loop {
        arch::wfe();
    }
}

extern "C" {
    fn _start();
    pub static _end: [u8; 0];
}
