use core::marker::{Sync, Send};
use core::{future::Future, pin::Pin};
use core::task::{Context, Poll, Waker};
use core::sync::atomic::{Ordering, AtomicU64};

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::task::Wake;

use spin::Mutex;
use hashbrown::HashSet;

pub mod executor;

pub use executor::Executor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(u64);

impl TaskId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()> + Sync + Send>>,
    id: TaskId,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static + Sync + Send) -> Task {
        Task {
            future: Box::pin(future),
            id: TaskId::new(),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

pub struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<Mutex<HashSet<TaskId>>>,
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<Mutex<HashSet<TaskId>>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue.lock().insert(self.task_id);
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}