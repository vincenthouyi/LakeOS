#![feature(asm)]
#![feature(const_fn)]
#![feature(llvm_asm)]
#![feature(decl_macro)]
#![feature(arbitrary_enum_discriminant)]

#![no_std]

extern crate num_traits;
#[macro_use] extern crate num_derive;

pub mod debug_printer;
pub mod object;
pub mod syscall;
pub mod init;
pub mod error;
pub mod vspace;
pub mod thread;

pub use debug_printer::{kprint, kprintln};