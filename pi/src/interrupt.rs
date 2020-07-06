//use common::IO_BASE;
use volatile::prelude::*;
use volatile::{Volatile, ReadVolatile};

//const INT_BASE: usize = IO_BASE + 0xB000 + 0x200;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Interrupt {
    Timer1 = 1,
    Timer3 = 3,
    Usb = 9,
    Aux = 29,
    Gpio0 = 49,
    Gpio1 = 50,
    Gpio2 = 51,
    Gpio3 = 52,
    Uart = 57,
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    // FIXME: Fill me in.
    IRQBasicPending: ReadVolatile<u32>,
    IRQPending: [ReadVolatile<u32>; 2],
    FIQControl: Volatile<u32>,
    EnableIRQ: [Volatile<u32>; 2],
    EnableBasicIRQ: Volatile<u32>,
    DisableIRQ: [Volatile<u32>; 2],
    DisableBasicIRQ: Volatile<u32>,
}

/// An interrupt controller. Used to enable and disable interrupts as well as to
/// check if an interrupt is pending.
pub struct Controller {
    registers: &'static mut Registers
}

impl Controller {
    /// Returns a new handle to the interrupt controller.
    pub fn new(page_base: usize) -> Controller {
        Controller {
            registers: unsafe { &mut *(page_base as *mut Registers) },
        }
    }

    pub fn enable(&mut self, int: usize) {
        self.registers.EnableIRQ[int / 32].write(1 << (int % 32));
    }

    pub fn disable(&mut self, int: usize) {
        self.registers.DisableIRQ[int / 32].write(1 << (int % 32));
    }

    pub fn is_pending(&self, int: usize) -> bool {
        self.registers.IRQPending[int / 32].has_mask(1 << int % 32)
    }

    pub fn pending_irq(&self) -> usize {
        if self.registers.IRQBasicPending.has_mask(1 << 8) {
            self.registers.IRQPending[0].read().trailing_zeros() as usize
        } else if self.registers.IRQBasicPending.has_mask(1 << 9) {
            self.registers.IRQPending[1].read().trailing_zeros() as usize
        } else {
            panic!("spurious interrupt!");
        }
    }
}
