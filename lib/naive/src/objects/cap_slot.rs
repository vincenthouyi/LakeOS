use crate::space_manager::gsm;
use core::ops::Drop;

#[derive(Debug)]
pub struct CapSlot(usize);

impl CapSlot {
    pub fn new(inner: usize) -> Self {
        Self(inner)
    }

    pub fn slot(&self) -> usize {
        self.0
    }
}

impl Drop for CapSlot {
    fn drop(&mut self) {
        if self.0 != 0 {
            gsm!().cspace_free(self.0);
        }
    }
}
