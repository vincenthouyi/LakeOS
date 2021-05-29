use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::ops::Drop;

use crate::ep_server::{EpServer};
use crate::ipc::FaultMessage;
use crate::objects::{EpCap, ReplyCap};
use crate::Result;

#[derive(Clone)]
pub struct FaultReceiver {
    badge: usize,
    ep_server: &'static EpServer
}

impl FaultReceiver {
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
    inner: &'a FaultReceiver,
}

impl<'a> RecvFuture<'a> {
    pub fn new(inner: &'a FaultReceiver) -> Self {
        Self { inner }
    }
}

impl<'a> Future for RecvFuture<'a> {
    type Output = Result<(FaultMessage, ReplyCap)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let badge = self.inner.badge;
        let msg_handlers = self.inner.ep_server.get_fault_handlers();
        let handler = msg_handlers.get(&badge).unwrap();
        if let Ok(msg) = handler.buf.pop() {
            Poll::Ready(Ok(msg))
        } else {
            handler.waker.push(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl Drop for FaultReceiver {
    fn drop(&mut self) {
        self.ep_server.remove_fault_handler(self.badge);
    }
}