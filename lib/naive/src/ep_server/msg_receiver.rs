use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use core::ops::Drop;
use crossbeam_queue::{ArrayQueue, SegQueue};
use alloc::sync::Arc;

use crate::ep_server::{EpServer, MessageHandler};
use crate::ipc::Message;
use crate::objects::EpCap;
use crate::Result;

struct MsgHandler {
    waker: Arc<SegQueue<Waker>>,
    buf: Arc<ArrayQueue<Message>>,
}

impl MessageHandler for MsgHandler {
    fn handle_message(&self, _ep_server: &EpServer, _badge: usize, msg: Message) {
        self.buf.push(msg).unwrap();
        while let Ok(waker) = self.waker.pop() {
            waker.wake();
        }
    }
}

pub struct MsgReceiver {
    badge: usize,
    ep_server: &'static EpServer,
    waker: Arc<SegQueue<Waker>>,
    buf: Arc<ArrayQueue<Message>>,
}

impl MsgReceiver {
    pub fn new(ep_server: &'static EpServer) -> Self {
        let waker = Arc::new(SegQueue::new());
        let buf = Arc::new(ArrayQueue::new(10));
        let handler = MsgHandler { waker: waker.clone(), buf: buf.clone() };
        let badged_ep = ep_server.handle_message(handler).unwrap();
        Self {
            badge: badged_ep.badge(), ep_server, waker, buf
        }
    }

    pub fn receive<'a>(&'a self) -> RecvFuture<'a> {
        RecvFuture::new(self)
    }

    pub fn badged_ep(&self) -> EpCap {
        self.ep_server.get_badged_ep(self.badge)
    }
}

pub struct RecvFuture<'a> {
    inner: &'a MsgReceiver,
}

impl<'a> RecvFuture<'a> {
    pub fn new(inner: &'a MsgReceiver) -> Self {
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

impl Drop for MsgReceiver {
    fn drop(&mut self) {
        self.ep_server.remove_message_handler(self.badge);
    }
}