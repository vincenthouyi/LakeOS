#![no_std]
#![no_main]

#![feature(once_cell)]

extern crate alloc;

#[macro_use] extern crate naive;

use rustyl4api::{kprintln};
use naive::ep_server::EP_SERVER;
use naive::ns::ns_client;

mod shell;

#[naive::main]
async fn main() -> () {
    use crate::alloc::string::ToString;
    kprintln!("shell process start");

    let mut stdio_cap_slot = None;
    
    while let None = stdio_cap_slot {
        stdio_cap_slot = ns_client()
            .lock()
            .lookup_service("tty".to_string())
            .await
            .ok();
    }
    unsafe {
        naive::io::stdio::STDOUT_CAP = stdio_cap_slot.unwrap();
        naive::io::stdio::STDIN_CAP = stdio_cap_slot.unwrap();
    }

    loop {
        shell::shell("test shell >").await;
        println!("Test shell exit, restarting...").await;
    }
}