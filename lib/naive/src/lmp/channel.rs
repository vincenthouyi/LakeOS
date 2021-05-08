use alloc::{sync::Arc, vec::Vec};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use futures_util::stream::Stream;
use spin::Mutex;

use crate::{
    ep_server::{EpMsgHandler, EpServer, EP_SERVER},
    space_manager::{gsm, copy_cap},
    objects::{EpCap, RamObj, CapSlot},
    ipc,
    Result
};

use super::{ArgumentBuffer, LmpMessage};

pub struct LmpChannel {
    remote_ntf_ep: EpCap,
    local_ntf_badge: usize,
    argbuf: ArgumentBuffer,
    role: Role,
}

pub enum Role {
    Server,
    Client,
}

impl LmpChannel {
    pub fn new(
        remote_ntf_ep: EpCap,
        local_ntf_badge: usize,
        argbuf: ArgumentBuffer,
        role: Role,
    ) -> Self {
        Self {
            remote_ntf_ep,
            local_ntf_badge,
            argbuf,
            role,
        }
    }

    pub fn connect(server_ep: &EpCap, ntf_ep: EpCap, local_ntf_badge: usize) -> Result<Self> {
        use crate::objects::ReplyCap;
        use rustyl4api::vspace::Permission;

        /* Connect by sending client notification ep */
        let ret = server_ep.call(&[], Some(ntf_ep.into_slot())).unwrap();
        let msg = ret.into_message().unwrap();
        let svr_ntf_ep = EpCap::new(msg.cap_transfer.unwrap());

        /* Generate buffer cap and Derive a copy of buffer cap */
        let buf_cap = gsm!().alloc_object::<RamObj>(12).unwrap();
        let copied_cap = copy_cap(&buf_cap).unwrap();

        /* service event notification */
        let buf_ptr = gsm!().insert_ram_at(buf_cap, 0, Permission::writable());
        let argbuf = unsafe { ArgumentBuffer::new(buf_ptr as *mut usize, 4096) };

        /* send buffer cap to server */
        let reply_cap = ReplyCap::new(CapSlot::new(0));
        reply_cap.reply(&[], Some(copied_cap.into_slot())).unwrap();

        Ok(Self::new(
            svr_ntf_ep,
            local_ntf_badge,
            argbuf,
            Role::Client,
        ))
    }

    fn send_channel(&mut self) -> &mut [u8] {
        let argbuf_size = self.argbuf.len();
        if let Role::Server = self.role {
            &mut self.argbuf[0..argbuf_size / 2]
        } else {
            &mut self.argbuf[argbuf_size / 2..]
        }
    }

    fn recv_channel(&mut self) -> &mut [u8] {
        let argbuf_size = self.argbuf.len();
        if let Role::Client = self.role {
            &mut self.argbuf[0..argbuf_size / 2]
        } else {
            &mut self.argbuf[argbuf_size / 2..]
        }
    }

    fn send_message(&mut self, msg: &mut LmpMessage) {
        //TODO: handle msg > 2048. now panics.
        let chan = self.send_channel();
        chan[0] = 1;
        chan[1] = msg.msg.len() as u8;
        chan[2] = (msg.msg.len() >> 8) as u8;
        chan[3..3 + msg.msg.len()].copy_from_slice(&msg.msg);
        let cap_slot = msg.caps.pop();
        self.remote_ntf_ep
            .send(&[], cap_slot)
            .unwrap();
    }

    fn recv_message(&mut self) -> Option<LmpMessage> {
        let chan = self.recv_channel();
        if chan[0] == 0 {
            return None;
        }
        let arglen = ((chan[2] as usize) << 8) | chan[1] as usize;
        let msg = LmpMessage {
            msg: chan[3..3 + arglen].to_vec(),
            caps: Vec::new(),
        };
        chan[0] = 0;
        Some(msg)
    }

    pub fn can_send(&mut self) -> bool {
        self.send_channel()[0] == 0
    }

    pub fn can_recv(&mut self) -> bool {
        self.recv_channel()[0] == 0
    }

    pub fn notification_badge(&self) -> usize {
        self.local_ntf_badge
    }
}

#[derive(Clone)]
pub struct LmpChannelHandle {
    pub inner: Arc<Mutex<LmpChannel>>,
    pub waker: Arc<Mutex<Vec<Waker>>>,
    pub rx_queue: Arc<Mutex<Vec<LmpMessage>>>,
}

impl LmpChannelHandle {
    pub fn new(
        remote_ntf_ep: EpCap,
        local_ntf_badge: usize,
        argbuf: ArgumentBuffer,
        role: Role,
    ) -> Self {
        let inner = LmpChannel::new(remote_ntf_ep, local_ntf_badge, argbuf, role);
        Self::from_inner(inner)
    }

    pub fn from_inner(inner: LmpChannel) -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner)),
            waker: Arc::new(Mutex::new(Vec::new())),
            rx_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn connect(server_ep: &EpCap, ntf_ep: EpCap, ntf_badge: usize) -> Result<Self> {
        let inner = LmpChannel::connect(server_ep, ntf_ep, ntf_badge)?;
        let chan = Self::from_inner(inner);
        EP_SERVER.insert_event(ntf_badge, chan.clone());
        Ok(chan)
    }

    pub fn disconnect(&self) {
        let badge = self.inner.lock().notification_badge();
        EP_SERVER.remove_event(badge);
    }

    pub fn send_message(&self, msg: &mut LmpMessage) {
        self.inner.lock().send_message(msg)
    }

    pub fn poll_send<'a>(&'a self, msg: &'a mut LmpMessage) -> SendFuture<'a> {
        SendFuture::new(self, msg)
    }

    pub fn poll_recv(&self) -> RecvFuture<'_> {
        RecvFuture::new(self)
    }

    pub fn messages(&self) -> MessagesFuture<'_> {
        MessagesFuture::new(self)
    }
}

impl EpMsgHandler for LmpChannelHandle {
    fn handle_ipc(&self, _ep_server: &EpServer, msg: ipc::Message) {
        let ipc::Message {
            payload: _,
            payload_len: _,
            need_reply: _,
            cap_transfer,
            badge: _,
        } = msg;
        {
            let mut chan = self.inner.lock();
            if let Some(mut msg) = chan.recv_message() {
                if let Some(cap) = cap_transfer {
                    msg.caps.push(cap);
                }
                self.rx_queue.lock().push(msg);
                while let Some(waker) = self.waker.lock().pop() {
                    waker.wake();
                }
            }
        }
    }
}

pub struct SendFuture<'a> {
    channel: &'a LmpChannelHandle,
    message: &'a mut LmpMessage,
}

impl<'a> SendFuture<'a> {
    pub fn new(channel: &'a LmpChannelHandle, message: &'a mut LmpMessage) -> Self {
        Self { channel, message }
    }
}

impl<'a> Future for SendFuture<'a> {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut chan = self.channel.inner.lock();
        if chan.can_send() {
            chan.send_message(&mut self.message);
            Poll::Ready(Ok(()))
        } else {
            self.channel.waker.lock().push(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub struct RecvFuture<'a> {
    channel: &'a LmpChannelHandle,
}

impl<'a> RecvFuture<'a> {
    pub fn new(channel: &'a LmpChannelHandle) -> Self {
        Self { channel }
    }
}

impl<'a> Future for RecvFuture<'a> {
    type Output = Result<LmpMessage>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(msg) = self.channel.rx_queue.lock().pop() {
            Poll::Ready(Ok(msg))
        } else {
            self.channel.waker.lock().push(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub struct MessagesFuture<'a>(&'a LmpChannelHandle);

impl<'a> MessagesFuture<'a> {
    pub fn new(inner: &'a LmpChannelHandle) -> Self {
        Self(inner)
    }
}

impl<'a> Stream for MessagesFuture<'a> {
    type Item = LmpMessage;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0.poll_recv()).poll(cx).map(|r| r.ok())
    }
}
