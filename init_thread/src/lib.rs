#![feature(decl_macro)]
#![feature(asm)]
#![feature(const_fn)]

#![no_std]

extern crate alloc;
extern crate naive;

mod console;
mod gpio;
mod shell;
mod timer;

use rustyl4api::object::{Capability, EndpointObj};

use naive::space_manager::gsm;

mod prelude {
    pub use crate::console::{print, println};
}

static RPI3B_ELF: &'static [u8] = include_bytes!("../build/shell.elf");

use prelude::*;

static mut EP: Option<Capability<EndpointObj>> = None;

fn test_thread() -> ! {
    for i in 1..=1 {
        for _ in 0..10000000 {rustyl4api::syscall::nop()}
        println!("妈妈再爱我{}次", i);
    }

    let mut buf = [0usize; 5];
    let recved_buf = unsafe {
        EP.as_ref().unwrap().receive(&mut buf)
    }.unwrap();

    println!("receive buf {:?}", recved_buf);
    loop {}
}

fn spawn_test() {

    naive::thread::spawn(test_thread);

    let ep_cap = gsm!().alloc_object::<EndpointObj>(12)
                           .unwrap();

    unsafe {
        EP = Some(ep_cap);
    }

    let buf = [10usize, 11];
    println!("sending buf {:?}", buf);
    unsafe {
        EP.as_ref().unwrap().send(&buf).unwrap();
    }
    println!("after sending");
}

fn vm_test() {
    use alloc::vec::Vec;

    let mut vec = Vec::<usize>::new();

    for i in 0..512 {
        vec.push(i);
    }

    for (i, num) in vec.iter().enumerate() {
        rustyl4api::kprintln!("vec[{}]: {}", i, num);
    }
}

fn timer_test() {
    for i in 0..5 {
        println!("timer {}: {}", i, timer::current_time());
        timer::spin_sleep_ms(1000);
    }

    // works now, but we don't have interrupt handling at the moment
//    system_timer::tick_in(1000);
}

#[no_mangle]
pub fn main() {
    rustyl4api::kprintln!("Long may the sun shine!");

    gpio::init_gpio_server();

    console::init_console_server();

    timer::init_timer_server();

//    timer_test();

//    vm_test();

//    spawn_test();

    let ep = gsm!().alloc_object::<EndpointObj>(12).unwrap();
    naive::process::ProcessBuilder::new(&RPI3B_ELF)
        .stdin(ep.clone())
        .stdout(ep.clone())
        .stderr(ep.clone())
        .spawn()
        .expect("spawn process failed");

    loop {
        let mut buf = [0usize; 5];
        let testbuf = unsafe {
            core::slice::from_raw_parts(buf.as_ptr(), buf.len())
        };
        let recvbuf = ep.receive(&mut buf).unwrap();
        let c = core::char::from_u32(recvbuf[0] as u32).unwrap();
        print!("{}", c)
        // shell::shell("test shell >");
        // println!("Test shell exit, restarting...");
    }
}