#![no_std]
#![no_main]
#![feature(once_cell)]

extern crate alloc;

#[macro_use]
extern crate naive;

use naive::{fs::File, io};
use rustyl4api::kprintln;

mod shell;

#[naive::main]
async fn main() -> () {
    kprintln!("shell process start");

    let mut tty = None;
    while let None = tty {
        tty = File::open("/dev/tty").await.ok();
    }
    io::set_stdout(tty.unwrap());
    let tty = File::open("/dev/tty").await.unwrap();
    io::set_stdin(tty);

    loop {
        shell::shell("test shell >").await;
        println!("Test shell exit, restarting...").await;
    }
}
