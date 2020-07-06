#![no_std]
#![feature(lang_items)]

pub mod rt;

pub mod prelude {

}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
