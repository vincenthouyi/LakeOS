use core::cell::Cell;

use spin::Mutex;

use crate::prelude::*;

use crate::objects::{CNodeEntry, NullCap, EndpointCap};
use crate::plat::interrupt::Controller;

const NUM_IRQ: usize = 64;

pub struct InterruptController {
    IrqEp: [CNodeEntry; NUM_IRQ],
}

impl InterruptController {
    pub const fn new() -> Self {
        Self {
            IrqEp: [Cell::new(NullCap::mint()); NUM_IRQ]
        }
    }

    pub fn attach_irq(&mut self, irq: usize, ep: CNodeEntry) {
        self.IrqEp[irq] = ep;
    }

    pub fn receive_irq(&self) {
        let irq = Controller::new().pending_irq();

        Controller::new().disable(irq);

        EndpointCap::try_from(&self.IrqEp[irq])
            .expect("Receiving interrupt from unattached irq!")
            .do_set_signal(1 << irq);
    }

    pub fn listen_irq(&self, irq: usize) {
        Controller::new().enable(irq);
    }

    pub fn listen_irq_mask(&self, mask: u64) {
        Controller::new().listen_irq_mask(mask)
    }
}

pub static mut INTERRUPT_CONTROLLER: Mutex<InterruptController> = Mutex::new(InterruptController::new());