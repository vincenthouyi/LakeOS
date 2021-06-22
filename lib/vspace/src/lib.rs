#![no_std]
#![feature(llvm_asm)]
#![feature(bool_to_option)]
#![feature(const_fn_trait_bound)]

#[macro_use]
mod utils;

mod common;
pub use common::*;

mod addr;
pub use addr::{PhysAddr, VirtAddr};

pub mod arch;
pub use arch::{Table, Entry};

mod error;
pub use error::*;

pub mod permission;

pub mod mmu;

mod vspace;
pub use vspace::VSpace;

#[macro_use]
extern crate bitflags;

pub const PAGE_OFFSET: usize = 0xffff0000_00000000;