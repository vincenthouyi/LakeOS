use alloc::vec::Vec;

mod listener;
pub use listener::{LmpListener, LmpListenerHandle};

mod channel;
pub use channel::{LmpChannel, LmpChannelHandle, Role};

pub trait LmpHandler: Send + Sync {
    fn handle_message(&self, msg: LmpMessage);
}

#[derive(Default, Debug, Clone)]
pub struct LmpMessage {
    pub opcode: usize,
    pub msg: Vec<u8>,
    pub caps: Vec<usize>,
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
