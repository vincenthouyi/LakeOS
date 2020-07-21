#![no_std]
#![feature(lang_items)]
#![feature(decl_macro)]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]
#![feature(const_saturating_int_methods)]

pub mod rt;
pub mod io;
mod space_manager;
mod vm_allocator;
mod utils;

pub mod prelude {
    pub use crate::io::{print, println};
}

extern "C" {
    static _end: [u8; 0];
}