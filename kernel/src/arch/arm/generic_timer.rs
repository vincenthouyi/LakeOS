use crate::prelude::*;

const GEN_TIMER_REG_BASE: usize = KERNEL_OFFSET+ 0x40000000;

use pi::generic_timer;

pub struct Timer { inner: generic_timer::Timer }

impl Timer {
    pub fn new() -> Timer {
        Timer {
            inner: generic_timer::Timer::new(GEN_TIMER_REG_BASE)
        }
    }

//    pub fn read(&self) -> u64 {
//        self.read()
//    }

    pub fn tick_in(&mut self, us: u32) {
        self.inner.tick_in(us)
    }

    pub fn initialize(&mut self, cpu: usize) {
        self.inner.initialize(cpu)
    }

    pub fn is_pending(&self, cpu: usize) -> bool {
        self.inner.is_pending(cpu)
    }
}