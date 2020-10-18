use core::task::{Context, Poll, Waker};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use hashbrown::HashSet;
use spin::Mutex;
use crossbeam_queue::SegQueue;

use super::{Task, TaskId, TaskWaker};

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<SegQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(SegQueue::new()),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task_id, task).is_some() {
            panic!("task id already in tasks");
        }
        self.task_queue.push(task_id);
    }

    pub fn run_ready_tasks(&mut self) {
        use alloc::vec::Vec;

        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        while let Ok(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task no longer exists
            };
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task done -> remove it and its cached waker
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    pub fn run(&mut self) {
        loop {
            self.run_ready_tasks();
            // kprintln!("no ready tasks");
        }
    }
}