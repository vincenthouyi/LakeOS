#![no_std]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(decl_macro)]
#![feature(never_type)]
#![feature(llvm_asm)]

//#![cfg_attr(not(feature = "std"), no_std)]

//#[cfg(feature = "std")]
//extern crate core;
extern crate volatile;

pub mod common;
pub mod gpio;
pub mod timer;
pub mod uart;
//pub mod atags;
pub mod generic_timer;
pub mod interrupt;
