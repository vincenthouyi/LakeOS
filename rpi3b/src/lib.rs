#![no_std]

extern crate std;

#[no_mangle]
fn main() {
    loop {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
