use core::pin::Pin;
use core::task::{Poll, Context, Waker};
use core::future::Future;
use core::marker::PhantomData;

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

use super::{UrpcStreamChannel, Role, UrpcHandler};

pub struct UrpcListener<T> {
    listen_badge: usize,
    listen_ep: EpCap,
    accept_waker: Arc<Mutex<VecDeque<Waker>>>,
    backlog: Vec<T>,
    t: PhantomData<T>
}

impl<T: UrpcHandler> UrpcListener<T> {
    pub fn bind(listen_ep: EpCap, listen_badge: usize) -> Result<Self> {
        Ok(Self { listen_badge, listen_ep, backlog: Vec::new(),
                  accept_waker: Arc::new(Mutex::new(VecDeque::new())),
                  t: PhantomData})
    }

    pub fn accept_with(&self, c_ntf_ep: EpCap, s_ntf_ep: EpCap) -> Result<T> {
        use rustyl4api::vspace::Permission;

        let ret = self.listen_ep.reply_receive(&[], Some(s_ntf_ep.slot)).unwrap();

        let buf_cap = RamCap::new(s_ntf_ep.slot);
        let buf_ptr = gsm!().insert_ram_at(buf_cap.clone(), 0, Permission::writable());

        let channel = UrpcStreamChannel::new(
            Role::Server, c_ntf_ep, buf_cap, buf_ptr
        );

        while let Some(waker) = self.accept_waker.lock().pop_front() {
            waker.wake();
        }

        Ok(T::new(channel))
    }
}

#[derive(Clone)]
pub struct UrpcListenerHandle<T>(Arc<Mutex<UrpcListener<T>>>);

impl<T: UrpcHandler> UrpcListenerHandle<T> {
    pub fn from_listener(listener: UrpcListener<T>) -> Self {
        UrpcListenerHandle(Arc::new(Mutex::new(listener)))
    }

    pub fn incoming(&self) -> Incoming<T> { Incoming(self) }

    pub fn accept(&self) -> AcceptFuture<T> { AcceptFuture(self) }
}

impl<T: 'static + Clone + UrpcHandler + EpMsgHandler + Send + Sync> EpMsgHandler for UrpcListenerHandle<T> {
    fn handle_ipc(&self, ep_server: &EpServer, msg: IpcMessage, cap_transfer_slot: Option<usize>) {
        let c_ntf_cap = EpCap::new(cap_transfer_slot.unwrap());
        let (conn_badge, s_ntf_cap) = ep_server.derive_badged_cap().unwrap();
        let mut inner = self.0.lock();
        let stream = inner.accept_with(c_ntf_cap, s_ntf_cap).unwrap();
        inner.backlog.push(stream.clone());
        ep_server.insert_event(conn_badge, Box::new(stream));
    }
}

pub struct Incoming<'a, T>(&'a UrpcListenerHandle<T>);

impl<'a, T: UrpcHandler> Stream for Incoming<'a, T> {
    type Item = Result<T>;

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

pub struct AcceptFuture<'a, T>(&'a UrpcListenerHandle<T>);

impl<'a, T: UrpcHandler> Future for AcceptFuture<'a, T> {
    type Output = Result<T>;
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