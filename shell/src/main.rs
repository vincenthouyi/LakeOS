use rustyl4api::{kprintln};
use rustyl4api::object::{EpCap};
use rustyl4api::process::ProcessCSpace;

mod shell;

// #[no_mangle]
fn main() -> () {
    loop {
        shell::shell("test shell >");
        println!("Test shell exit, restarting...");
    }
}