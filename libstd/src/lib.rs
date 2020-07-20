#![no_std]
#![feature(lang_items)]
#![feature(decl_macro)]

pub mod rt;
pub mod io;

pub mod prelude {
    pub use crate::io::{print, println};
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
