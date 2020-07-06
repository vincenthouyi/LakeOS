#![feature(decl_macro)]
#![feature(optin_builtin_traits)]
#![feature(asm)]
#![feature(maybe_uninit_ref)]
#![feature(const_in_array_repeat_expressions)]
#![feature(const_generics)]
#![feature(const_int_pow)]
#![feature(llvm_asm)]

#![allow(non_snake_case)]

#![no_std]
#![allow(non_upper_case_globals)]

#[macro_use] extern crate num_derive;
#[macro_use] mod macros;
#[macro_use] mod console;
#[macro_use] mod arch;
mod plat;
mod objects;
#[macro_use] mod vspace;
mod syscall;
mod scheduler;
mod utils;
mod interrupt;

pub use scheduler::SCHEDULER;

pub const TICK: u32 = 2 * 100;
pub const TIME_SLICE: isize = 1000;

mod prelude {
    pub use rustyl4api as sysapi;
    pub use sysapi::error::{SysResult, SysError};
    pub use crate::console::{kprint, kprintln};
    pub use core::convert::TryFrom;

    pub const PHYS_BASE : usize = 0x80000;
    pub const KERNEL_BASE : usize = 0xffff0000_00080000;
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
    pub static _end: [u8; 0];
}
