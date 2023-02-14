// fn main() {
//     println!("Hello, world!");
// }

#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(proc_macro_hygiene)]
#![feature(asm_const)]

#[macro_export]
macro_rules! BIT {
    ($x:expr) => {
        1 << $x
    };
}

#[macro_export]
macro_rules! MASK {
    ($x:expr) => {
        BIT!($x) - 1
    };
}

#[macro_use]
mod console;
mod boot;
mod boot_info;
mod ram_block;
mod uart;

use log::error;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("Panic! {:?}", info);
    loop {
        //     arch::wfe();
    }
}

extern "C" {
    pub static _end: [u8; 0];
}
