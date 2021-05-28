use alloc::vec::Vec;

use crate::objects::CapSlot;
use crate::space_manager::gsm;

mod listener;
pub use listener::LmpListener;

mod channel;
pub use channel::{LmpChannel, Role};

pub trait LmpHandler: Send + Sync {
    fn handle_message(&self, msg: LmpMessage);
}

#[derive(Default, Debug)]
pub struct LmpMessage {
    pub msg: Vec<u8>,
    pub caps: Vec<CapSlot>,
}

#[derive(Clone)]
pub struct ArgumentBuffer {
    base_ptr: usize,
    buf_len: usize,
}

impl ArgumentBuffer {
    pub unsafe fn new(ptr: *mut usize, len: usize) -> Self {
        Self {
            base_ptr: ptr as usize,
            buf_len: len,
        }
    }
}

impl core::ops::Deref for ArgumentBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        use core::slice::from_raw_parts;

        unsafe { from_raw_parts(self.base_ptr as *const u8, self.buf_len) }
    }
}

impl core::ops::DerefMut for ArgumentBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        use core::slice::from_raw_parts_mut;

        unsafe { from_raw_parts_mut(self.base_ptr as *mut u8, self.buf_len) }
    }
}

impl core::ops::Drop for ArgumentBuffer {
    fn drop(&mut self) {
        gsm!().memory_unmap(self.base_ptr as *mut u8, self.buf_len)
    }
}
