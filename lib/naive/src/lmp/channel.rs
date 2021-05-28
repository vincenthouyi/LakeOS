use alloc::boxed::Box;
use alloc::vec::Vec;

use core::ops::Drop;
use core::pin::Pin;
use core::task::{Context, Poll};

use futures_util::future::BoxFuture;
use futures_util::ready;
use futures_util::stream::Stream;
use spin::Mutex;

use crate::{
    ep_receiver::EpReceiver,
    ep_server::EP_SERVER,
    objects::{EpCap, RamObj},
    space_manager::{copy_cap, gsm},
    Result,
};

use super::{ArgumentBuffer, LmpMessage};

pub struct LmpChannel {
    remote_ntf_ep: EpCap,
    receiver: EpReceiver,
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
        receiver: EpReceiver,
        argbuf: ArgumentBuffer,
        role: Role,
    ) -> Self {
        Self {
            remote_ntf_ep,
            receiver,
            argbuf,
            role,
        }
    }

    pub async fn connect(server_ep: &EpCap, receiver: EpReceiver) -> Result<Self> {
        use rustyl4api::vspace::Permission;

        /* Connect by sending client notification ep */
        let ntf_ep = copy_cap(&receiver.ep).unwrap();
        server_ep.send(&[], Some(ntf_ep.into_slot())).unwrap();

        let s_ntf_msg = receiver.receive().await.unwrap();
        let svr_ntf_ep = EpCap::new(s_ntf_msg.cap_transfer.unwrap());

        /* Generate buffer cap and Derive a copy of buffer cap */
        let buf_cap = gsm!().alloc_object::<RamObj>(12).unwrap();
        let copied_cap = copy_cap(&buf_cap).unwrap();

        /* service event notification */
        let buf_ptr = gsm!().insert_ram_at(buf_cap, 0, Permission::writable());
        let argbuf = unsafe { ArgumentBuffer::new(buf_ptr as *mut usize, 4096) };

        /* send buffer cap to server */
        svr_ntf_ep.send(&[], Some(copied_cap.into_slot())).unwrap();

        Ok(Self::new(svr_ntf_ep, receiver, argbuf, Role::Client))
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
        self.remote_ntf_ep.send(&[], cap_slot).unwrap();
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
        self.receiver.badge
    }
}

pub struct LmpChannelHandle {
    pub inner: Mutex<LmpChannel>,
}

impl LmpChannelHandle {
    pub fn new(
        remote_ntf_ep: EpCap,
        receiver: EpReceiver,
        argbuf: ArgumentBuffer,
        role: Role,
    ) -> Self {
        let inner = LmpChannel::new(remote_ntf_ep, receiver, argbuf, role);
        Self::from_inner(inner)
    }

    pub fn from_inner(inner: LmpChannel) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }

    pub async fn connect(server_ep: &EpCap, receiver: EpReceiver) -> Result<Self> {
        let inner = LmpChannel::connect(server_ep, receiver).await?;
        let chan = Self::from_inner(inner);
        Ok(chan)
    }

    pub fn disconnect(&self) {
        let badge = self.inner.lock().notification_badge();
        EP_SERVER.remove_event(badge);
    }

    pub fn send_message(&self, msg: &mut LmpMessage) {
        self.inner.lock().send_message(msg)
    }

    pub async fn poll_send<'a>(&'a self, msg: &'a mut LmpMessage) -> Result<()> {
        let mut inner = self.inner.lock();

        assert!(inner.can_send());

        inner.send_message(msg);

        Ok(())
    }

    pub async fn poll_recv(&self) -> Result<LmpMessage> {
        let mut inner = self.inner.lock();
        let ep_msg = inner.receiver.receive().await.unwrap();
        let mut msg = inner.recv_message().unwrap();
        if let Some(cap) = ep_msg.cap_transfer {
            msg.caps.push(cap);
        }
        Ok(msg)
    }

    pub fn messages(&self) -> MessagesFuture<'_> {
        MessagesFuture::new(self)
    }
}

pub struct MessagesFuture<'a> {
    channel: &'a LmpChannelHandle,
    recv_state: Option<BoxFuture<'a, LmpMessage>>,
}

impl<'a> MessagesFuture<'a> {
    pub fn new(channel: &'a LmpChannelHandle) -> Self {
        Self {
            channel,
            recv_state: None,
        }
    }
}

impl<'a> Stream for MessagesFuture<'a> {
    type Item = LmpMessage;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Self {
            channel,
            recv_state,
        } = &mut *self;
        let fut = recv_state.get_or_insert_with(|| {
            let channel = *channel;
            let fut = || async move { channel.poll_recv().await.unwrap() };
            Box::pin(fut())
        });

        let msg = ready!(fut.as_mut().poll(cx));

        recv_state.take();
        Poll::Ready(Some(msg))
    }
}

impl Drop for LmpChannelHandle {
    fn drop(&mut self) {
        self.disconnect();
    }
}
