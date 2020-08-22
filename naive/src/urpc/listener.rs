use core::pin::Pin;
use core::task::{Poll, Context, Waker};
use core::future::Future;

use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::collections::VecDeque;
use alloc::boxed::Box;

use spin::Mutex;

use rustyl4api::object::{EpCap, RamCap};
use rustyl4api::ipc::IpcMessage;

use futures_util::stream::Stream;

use crate::space_manager::gsm;
use crate::io::Result;
use crate::ep_server::{EpServer, EpMsgHandler};

use super::{UrpcStreamChannel, UrpcStream, UrpcStreamHandle, Role};


pub struct UrpcListener {
    listen_badge: usize,
    listen_ep: EpCap,
    accept_waker: Arc<Mutex<VecDeque<Waker>>>,
    backlog: Vec<UrpcStreamHandle>,
}

impl UrpcListener {
    pub fn bind(listen_ep: EpCap, listen_badge: usize) -> Result<Self> {
        Ok(Self { listen_badge, listen_ep, backlog: Vec::new(),
                  accept_waker: Arc::new(Mutex::new(VecDeque::new())) })
    }

    pub fn accept_with(&self, c_ntf_ep: EpCap, s_ntf_ep: EpCap) -> Result<UrpcStreamChannel> {
        use rustyl4api::vspace::Permission;

        let ret = self.listen_ep.reply_receive(&[], Some(s_ntf_ep.slot)).unwrap();

        let buf_cap = RamCap::new(s_ntf_ep.slot);
        let buf_ptr = gsm!().insert_ram_at(buf_cap.clone(), 0, Permission::writable());

        let stream = UrpcStreamChannel::new(
            Role::Server, c_ntf_ep, buf_cap, buf_ptr
        );

        while let Some(waker) = self.accept_waker.lock().pop_front() {
            waker.wake();
        }

        Ok(stream)
    }
}

#[derive(Clone)]
pub struct UrpcListenerHandle(Arc<Mutex<UrpcListener>>);

impl UrpcListenerHandle {
    pub fn from_listener(listener: UrpcListener) -> Self {
        UrpcListenerHandle(Arc::new(Mutex::new(listener)))
    }

    pub fn incoming(&self) -> Incoming { Incoming(self) }

    pub fn accept(&self) -> AcceptFuture { AcceptFuture(self) }
}

impl EpMsgHandler for UrpcListenerHandle {
    fn handle_ipc(&self, ep_server: &EpServer, msg: IpcMessage, cap_transfer_slot: Option<usize>) {
        let c_ntf_cap = EpCap::new(cap_transfer_slot.unwrap());
        let (conn_badge, s_ntf_cap) = ep_server.derive_badged_cap().unwrap();
        let mut inner = self.0.lock();
        let stream_inner = inner.accept_with(c_ntf_cap, s_ntf_cap).unwrap();
        let stream = UrpcStream::from_stream(stream_inner);
        let stream = UrpcStreamHandle::from_stream(stream);
        inner.backlog.push(stream.clone());
        crate::ep_server::ep_server().insert_event(conn_badge, Box::new(stream));
    }
}

pub struct Incoming<'a>(&'a UrpcListenerHandle);

impl<'a> Stream for Incoming<'a> {
    type Item = Result<UrpcStreamHandle>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut inner = self.0.0.lock();
        if let Some(stream) = inner.backlog.pop() {
            Poll::Ready(Some(Ok(stream)))
        } else {
            inner.accept_waker.lock().push_back(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub struct AcceptFuture<'a>(&'a UrpcListenerHandle);

impl<'a> Future for AcceptFuture<'a> {
    type Output = Result<UrpcStreamHandle>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut inner = self.0.0.lock();
        if let Some(stream) = inner.backlog.pop() {
            Poll::Ready(Ok(stream))
        } else {
            inner.accept_waker.lock().push_back(cx.waker().clone());
            Poll::Pending
        }
    }
}