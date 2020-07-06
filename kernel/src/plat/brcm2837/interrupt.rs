use pi::interrupt::Controller as IrqCntl;

const INT_BASE: usize = crate::prelude::IO_BASE + 0xB000 + 0x200;

pub struct Controller { }

impl Controller {
    pub fn new() -> Controller {
        Controller { }
    }

    pub fn enable(&mut self, int: usize) {
        IrqCntl::new(INT_BASE).enable(int)
    }

    pub fn disable(&mut self, int: usize) {
        IrqCntl::new(INT_BASE).disable(int)
    }

//    pub fn is_pending(&self, int: usize) -> bool {
//        IrqCntl::new(INT_BASE).is_pending(int)
//    }

    pub fn pending_irq(&self) -> usize {
        IrqCntl::new(INT_BASE).pending_irq()
    }
}
