use core::task::{Context, Poll};
use core::task::{Waker, RawWaker};
use core::task::RawWakerVTable;
use alloc::collections::VecDeque;
use crossbeam_queue::SegQueue;

use super::Task;

pub struct Executor {
    task_queue: SegQueue<Task>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            task_queue: SegQueue::new(),
        }
    }

    pub fn spawn(&self, task: Task) {
        self.task_queue.push(task)
    }

    pub fn run(&self) {
        while let Ok(mut task) = self.task_queue.pop() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {} // task done
                Poll::Pending => self.task_queue.push(task),
            }
        }
    }
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}
