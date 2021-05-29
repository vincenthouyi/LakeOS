use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::ops::Drop;

use crate::ep_server::{EpServer};
use crate::ipc::Message;
use crate::objects::EpCap;
use crate::Result;

pub struct MsgReceiver {
    badge: usize,
    ep_server: &'static EpServer
}

impl MsgReceiver {
    pub fn new(badge: usize, ep_server: &'static EpServer) -> Self {
        Self {
            badge, ep_server
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
        let badge = self.inner.badge;
        let msg_handlers = self.inner.ep_server.get_msg_handlers();
        let handler = msg_handlers.get(&badge).unwrap();
        if let Ok(msg) = handler.buf.pop() {
            Poll::Ready(Ok(msg))
        } else {
            handler.waker.push(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl Drop for MsgReceiver {
    fn drop(&mut self) {
        self.ep_server.remove_message_handler(self.badge);
    }
}