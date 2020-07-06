#![feature(asm)]
#![feature(const_fn)]
#![feature(llvm_asm)]
#![no_std]

extern crate num_traits;
#[macro_use] extern crate num_derive;

pub mod object;
pub mod syscall;
pub mod init;
pub mod error;
pub mod vspace;
