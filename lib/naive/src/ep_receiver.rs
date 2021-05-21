use core::task::{Waker, Poll, Context};
use core::future::Future;
use core::pin::Pin;

use alloc::sync::Arc;

use crossbeam_queue::{SegQueue, ArrayQueue};

use crate::ipc::Message;
use crate::objects::EpRef;
use crate::ep_server::{EpServer, EpMsgHandler};
use crate::Result;

#[derive(Clone)]
pub struct EpReceiver {
    pub ep: EpRef,
    pub badge: usize,
    waker: Arc<SegQueue<Waker>>,
    buf: Arc<ArrayQueue<Message>>,
}

impl EpReceiver {
    pub fn new(ep: EpRef, badge: usize) -> Self {
        Self {
            ep,
            badge,
            waker: Arc::new(SegQueue::new()),
            buf: Arc::new(ArrayQueue::new(10)),
        }
    }

    pub fn receive<'a>(&'a self) -> RecvFuture<'a> {
        RecvFuture::new(self)
    }
}

impl EpMsgHandler for EpReceiver {
    fn handle_ipc(&self, _ep_server: &EpServer, msg: Message) {
        self.buf.push(msg).unwrap();
        while let Ok(waker) = self.waker.pop() {
            waker.wake();
        }
    }
}

pub struct RecvFuture<'a> {
    inner: &'a EpReceiver,
}

impl<'a> RecvFuture<'a> {
    pub fn new(inner: &'a EpReceiver) -> Self {
        Self { inner }
    }
}

impl<'a> Future for RecvFuture<'a> {
    type Output = Result<Message>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Ok(msg) = self.inner.buf.pop() {
            Poll::Ready(Ok(msg))
        } else {
            self.inner.waker.push(cx.waker().clone());
            Poll::Pending
        }
    }
}
