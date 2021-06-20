#![no_std]
#![feature(llvm_asm)]
#![feature(bool_to_option)]
#![feature(const_fn_trait_bound)]

#[macro_use]
mod utils;

pub mod common;

mod addr;
pub use addr::{PhysAddr, VirtAddr};

mod arch;
pub use arch::*;

mod error;
pub use error::*;

pub mod permission;

mod vspace;
pub use vspace::VSpace;

#[macro_use]
extern crate bitflags;

pub const PAGE_OFFSET: usize = 0;