#![no_std]
#![no_main]

extern crate alloc;

#[macro_use] extern crate naive;

use rustyl4api::{kprintln};
use naive::io::{stdin, stdout};

mod shell;

#[naive::main]
async fn main() -> () {
    kprintln!("shell process start");

    stdin();
    stdout();

    loop {
        shell::shell("test shell >").await;
        println!("Test shell exit, restarting...").await;
    }
}