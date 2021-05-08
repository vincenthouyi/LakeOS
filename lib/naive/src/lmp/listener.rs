use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use alloc::{collections::VecDeque, sync::Arc};

use spin::Mutex;

use futures_util::stream::Stream;

use crate::{
    ep_server::{EpMsgHandler, EpServer},
    space_manager::{gsm, copy_cap},
    objects::{EpCap, RamCap, EpRef},
    ipc,
    Result,
};

use super::{ArgumentBuffer, LmpChannel, LmpChannelHandle, Role};

#[allow(dead_code)]
pub struct LmpListener {
    badge: usize,
    listen_ep: EpRef,
}

impl LmpListener {
    pub fn new(listen_ep: EpRef, badge: usize) -> Self {
        Self {
            badge,
            listen_ep,
        }
    }

    pub fn accept_with(
        &self,
        c_ntf_ep: EpCap,
        s_ntf_ep: EpCap,
        s_ntf_badge: usize,
    ) -> Result<LmpChannel> {
        use rustyl4api::vspace::Permission;

        let ret = self
            .listen_ep
            .reply_receive(&[], Some(s_ntf_ep.into_slot()))
            .unwrap()
            .into_message()
            .unwrap();

        let buf_cap = RamCap::new(ret.cap_transfer.unwrap());
        let buf_ptr = gsm!().insert_ram_at(buf_cap, 0, Permission::writable());

        let argbuf = unsafe { ArgumentBuffer::new(buf_ptr as *mut usize, 4096) };

        Ok(LmpChannel::new(
            c_ntf_ep,
            s_ntf_badge,
            argbuf,
            Role::Server,
        ))
    }

    pub fn copy_connector_ep(&self) -> Option<EpCap> {
        copy_cap(&self.listen_ep)
    }
}

#[derive(Clone)]
pub struct LmpListenerHandle {
    inner: Arc<Mutex<LmpListener>>,
    backlog: Arc<Mutex<VecDeque<LmpChannelHandle>>>,
    waker: Arc<Mutex<VecDeque<Waker>>>,
}

impl LmpListenerHandle {
    pub fn new(listen_ep: EpRef, badge: usize) -> Self {
        let inner = LmpListener::new(listen_ep, badge);
        Self::from_inner(inner)
    }

    pub fn from_inner(inner: LmpListener) -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner)),
            backlog: Arc::new(Mutex::new(VecDeque::new())),
            waker: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn poll_accept(&mut self) -> AcceptFuture {
        AcceptFuture(self)
    }

    pub fn incoming(&mut self) -> IncomingFuture {
        IncomingFuture(self)
    }

    pub fn derive_connector_ep(&self) -> Option<EpCap> {
        self.inner.lock().copy_connector_ep()
    }
}

impl EpMsgHandler for LmpListenerHandle {
    fn handle_ipc(&self, ep_server: &EpServer, msg: ipc::Message) {
        let ipc::Message {
            payload: _,
            payload_len: _,
            need_reply: _,
            cap_transfer,
            badge: _,
        } = msg;
        {
            let c_ntf_cap = EpCap::new(cap_transfer.unwrap());
            let (conn_badge, s_ntf_cap) = ep_server.derive_badged_cap().unwrap();
            let inner = self.inner.lock();
            let chan = inner.accept_with(c_ntf_cap, s_ntf_cap, conn_badge).unwrap();
            let chan = LmpChannelHandle::from_inner(chan);
            ep_server.insert_event(conn_badge, chan.clone());
            self.backlog.lock().push_back(chan.clone());
            while let Some(waker) = self.waker.lock().pop_front() {
                waker.wake()
            }
        }
    }
}

pub struct AcceptFuture<'a>(&'a mut LmpListenerHandle);

impl<'a> Future for AcceptFuture<'a> {
    type Output = Result<LmpChannelHandle>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let income = self.0.backlog.lock().pop_front();
        match income {
            Some(server) => Poll::Ready(Ok(server)),
            None => {
                self.0.waker.lock().push_back(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

pub struct IncomingFuture<'a>(&'a mut LmpListenerHandle);

impl<'a> Stream for IncomingFuture<'a> {
    type Item = Result<LmpChannelHandle>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0.poll_accept())
            .poll(cx)
            .map(|r| Some(r))
    }
}
