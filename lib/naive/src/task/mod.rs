use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll, Waker};
use core::{future::Future, pin::Pin};

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::task::Wake;

use conquer_once::spin::OnceCell;
use crossbeam_queue::SegQueue;

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
    future: Pin<Box<dyn Future<Output = ()>>>,
    id: TaskId,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
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
    task_queue: Arc<SegQueue<TaskId>>,
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<SegQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue.push(self.task_id);
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

//TODO: make global executor thread local
unsafe impl core::marker::Send for Executor {}
unsafe impl core::marker::Sync for Executor {}
pub fn global_executor() -> &'static Executor {
    static EXECUTOR: OnceCell<Executor> = OnceCell::uninit();

    EXECUTOR.try_get_or_init(|| Executor::new()).unwrap()
}

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    let task = Task::new(future);
    global_executor().spawn(task);
}
