#![no_std]

extern crate std;

use std::prelude::*;

#[no_mangle]
fn main() {
    println!("here");
    loop {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
