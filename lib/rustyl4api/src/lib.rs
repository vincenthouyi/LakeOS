#![feature(asm)]
#![feature(llvm_asm)]
#![feature(decl_macro)]
#![feature(arbitrary_enum_discriminant)]
#![no_std]

extern crate num_traits;
#[macro_use]
extern crate num_derive;

#[macro_use]
pub mod debug_printer;
pub mod error;
pub mod fault;
pub mod init;
pub mod ipc;
pub mod objects;
pub mod process;
pub mod syscall;
pub mod thread;
pub(crate) mod utils;
pub mod vspace;
