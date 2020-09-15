use core::marker::{Sync, Send};
use core::{future::Future, pin::Pin};
use core::task::{Context, Poll};
use alloc::boxed::Box;

pub mod executor;

pub use executor::Executor;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()> + Sync + Send>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static + Sync + Send) -> Task {
        Task {
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}