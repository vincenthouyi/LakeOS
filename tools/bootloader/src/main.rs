// fn main() {
//     println!("Hello, world!");
// }

#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(naked_functions)]
#![feature(asm)]
#![feature(proc_macro_hygiene)]

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

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    kprintln!("Panic! {:?}", info);
    loop {
        //     arch::wfe();
    }
}

extern "C" {
    pub static _end: [u8; 0];
}
