#![feature(asm)]
#![feature(decl_macro)]
#![feature(alloc_error_handler)]
#![feature(const_in_array_repeat_expressions)]
#![feature(const_fn)]
#![feature(allocator_api)]
// #![feature(const_saturating_int_methods)]
#![feature(linked_list_cursors)]
#![feature(llvm_asm)]
#![feature(wake_trait)]
#![feature(extend_one)]
#![feature(toowned_clone_into)]
#![feature(str_internals)]
#![feature(shrink_to)]
#![no_std]

extern crate alloc;
#[macro_use]
extern crate rustyl4api;
#[macro_use]
extern crate futures_util;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate async_trait;

#[macro_use]
mod utils;
pub mod ep_server;
pub mod io;
pub mod lmp;
mod macros;
pub mod ns;
mod panic;
pub mod process;
pub mod rpc;
pub mod rt;
pub mod space_manager;
pub mod stream;
pub mod task;
pub mod thread;
pub mod time;
mod vm_allocator;
pub mod fs;
pub mod path;
pub mod os_str;
mod os_str_bytes;

pub use naive_attributes::main;

extern "C" {
    static _end: [u8; 0];
}
