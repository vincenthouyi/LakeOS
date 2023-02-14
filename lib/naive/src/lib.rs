#![feature(decl_macro)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(linked_list_cursors)]
#![feature(extend_one)]
#![feature(str_internals)]
#![feature(exact_size_is_empty)]
#![feature(utf8_chunks)]
#![no_std]

extern crate alloc;
#[macro_use]
extern crate rustyl4api;
extern crate futures_util;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate lazy_static;

#[macro_use]
mod utils;
pub mod ep_server;
pub mod error;
pub mod fs;
pub mod io;
pub mod ipc;
pub mod lmp;
mod macros;
pub mod ns;
pub mod objects;
pub mod os_str;
mod os_str_bytes;
mod panic;
pub mod path;
pub mod process;
pub mod rpc;
pub mod rt;
pub mod space_manager;
mod spaceman;
pub mod task;
pub mod thread;
pub mod time;
mod vm_allocator;

pub use error::{Error, Result};

pub use naive_attributes::main;

extern "C" {
    static _end: [u8; 0];
}
