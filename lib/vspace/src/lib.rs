#![no_std]
#![feature(llvm_asm)]
#![feature(bool_to_option)]
#![feature(const_fn_trait_bound)]
#![feature(specialization)]

#![allow(incomplete_features)]

#[macro_use]
mod utils;

mod common;
pub use common::*;

mod addr;
pub use addr::{PhysAddr, VirtAddr};

pub mod arch;

mod error;
pub use error::*;

pub mod permission;

pub mod mmu;

mod page_table;
pub use page_table::{Table, Entry};

mod vspace;
pub use vspace::VSpace;

#[macro_use]
extern crate bitflags;
