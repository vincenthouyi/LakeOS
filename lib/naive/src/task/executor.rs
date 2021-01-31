use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::task::{Context, Poll, Waker};
use crossbeam_queue::SegQueue;
use spin::Mutex;

use super::{Task, TaskId, TaskWaker};

pub struct Executor {
    tasks: Mutex<BTreeMap<TaskId, Task>>,
    task_queue: Arc<SegQueue<TaskId>>,
    waker_cache: Mutex<BTreeMap<TaskId, Waker>>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            tasks: Mutex::new(BTreeMap::new()),
            task_queue: Arc::new(SegQueue::new()),
            waker_cache: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn spawn(&self, task: Task) {
        let task_id = task.id;
        if self.tasks.lock().insert(task_id, task).is_some() {
            panic!("task id already in tasks");
        }
        self.task_queue.push(task_id);
    }

    pub fn run_ready_tasks(&self) {
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        while let Ok(task_id) = task_queue.pop() {
            let mut task = match tasks.lock().remove(&task_id) {
                Some(task) => task,
                None => continue, // task no longer exists
            };
            let waker = waker_cache
                .lock()
                .remove(&task_id)
                .unwrap_or_else(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                }
                Poll::Pending => {
                    tasks.lock().insert(task_id, task);
                    waker_cache.lock().insert(task_id, waker);
                }
            }
        }
    }

    pub fn run(&self) {
        loop {
            self.run_ready_tasks();
            // kprintln!("no ready tasks");
        }
    }
}
