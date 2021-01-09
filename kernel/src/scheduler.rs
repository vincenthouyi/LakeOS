use crate::objects::TcbObj;
use crate::prelude::*;
use crate::utils::percore::PerCore;
use crate::utils::tcb_queue::TcbQueue;
use crate::NCPU;
use core::cell::UnsafeCell;

pub static SCHEDULER: PerCore<Scheduler, NCPU> = PerCore([UnsafeCell::new(Scheduler::new()); NCPU]);

#[derive(Debug)]
pub struct Scheduler {
    queue: TcbQueue,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            queue: TcbQueue::new(),
        }
    }

    pub fn push(&self, tcb: &TcbObj) {
        self.queue.enqueue(tcb)
    }

    pub fn head(&self) -> Option<&TcbObj> {
        self.queue.head()
    }

    pub fn head_mut(&self) -> Option<&mut TcbObj> {
        self.queue.head_mut()
    }

    pub fn pop(&self) -> Option<&'static mut TcbObj> {
        self.queue.dequeue()
    }

    pub fn activate(&self) -> ! {
        if let Some(head) = self.head() {
            if (head.timeslice()) == 0 {
                let tcb = self.pop().unwrap();
                self.push(tcb);
                // kprintln!("switching {:p} -> {:p}: {:x?}", tcb, self.head().unwrap(), self.head().unwrap());

                self.head()
                    .unwrap()
                    .set_timeslice(crate::TIME_SLICE as usize);
            }
            self.head_mut().unwrap().activate();
        } else {
            kprintln!("not schedulable TCB. wait for interrupt!");
            loop {
                crate::arch::wfe()
            }
        }
    }
}
