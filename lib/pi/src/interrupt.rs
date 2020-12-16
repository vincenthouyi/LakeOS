//use common::IO_BASE;
use volatile::Volatile;

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
    IRQBasicPending: u32,
    IRQPending: [u32; 2],
    FIQControl: u32,
    EnableIRQ: [u32; 2],
    EnableBasicIRQ: u32,
    DisableIRQ: [u32; 2],
    DisableBasicIRQ: u32,
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
        Volatile::new_write_only(&mut self.registers.EnableIRQ[int / 32])
            .write(1 << (int % 32));
    }

    pub fn disable(&mut self, int: usize) {
        Volatile::new_write_only(&mut self.registers.DisableIRQ[int / 32])
            .write(1 << (int % 32));
    }

    pub fn enable_mask(&mut self, mask: u64) {
        let lower = (mask as u32) & (!0u32);
        let higher = (mask >> 32) as u32;
        if lower != 0 {
            Volatile::new_write_only(&mut self.registers.EnableIRQ[0])
                .write(lower)
        }

        if higher != 0 {
            Volatile::new_write_only(&mut self.registers.EnableIRQ[1])
                .write(higher)
        }
    }

    pub fn is_pending(&self, int: usize) -> bool {
        let val = Volatile::new_read_only(&self.registers.IRQPending[int / 32]).read();
        val & 1 << int % 32 == 1 << int % 32
    }

    pub fn pending_irq(&self) -> usize {
        if Volatile::new_read_only(&self.registers.IRQBasicPending).read() & (1 << 8) != 0 {
            Volatile::new_read_only(&self.registers.IRQPending[0])
                .read()
                .trailing_zeros() as usize
        } else if Volatile::new_read_only(&self.registers.IRQBasicPending).read() & (1 << 9) != 0 {
            Volatile::new_read_only(&self.registers.IRQPending[1])
                .read()
                .trailing_zeros() as usize
        } else {
            panic!("spurious interrupt!");
        }
    }
}
