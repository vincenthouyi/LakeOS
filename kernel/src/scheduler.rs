use crate::prelude::*;
use crate::objects::TcbObj;
use crate::utils::tcb_queue::TcbQueue;
use crate::utils::percore::PerCore;

pub static SCHEDULER: PerCore<Scheduler, 1> = PerCore([Scheduler::new(); 1]);

#[derive(Debug)]
pub struct Scheduler {
    queue: TcbQueue,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {queue: TcbQueue::new() }
    }

    pub fn init(&mut self) {
        self.queue = TcbQueue::new();
    }

    pub fn push(&self, tcb: &mut TcbObj) {
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
            if (head.time_slice) == 0 {
                let tcb = self.pop().unwrap();
                self.push(tcb);
                self.head_mut().unwrap().time_slice = crate::TIME_SLICE as usize;
            }
            self.head_mut().unwrap().activate();
        } else {
            kprintln!("not schedulable TCB. wait for interrupt!");
            loop { crate::arch::wfe() }
        }
    }
}
