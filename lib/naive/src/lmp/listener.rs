use core::{
    pin::Pin,
    task::{Context, Poll},
};

use alloc::boxed::Box;
use alloc::sync::Arc;

use futures_util::future::BoxFuture;
use futures_util::ready;
use futures_util::stream::Stream;

use crate::{
    ep_server::{EP_SERVER, MsgReceiver},
    objects::{EpCap, RamCap},
    space_manager::gsm,
    Result, Error,
};

use super::{ArgumentBuffer, LmpChannel, Role};

#[derive(Clone)]
pub struct LmpListener {
    receiver: Arc<MsgReceiver>,
}

impl LmpListener {
    pub fn new(receiver: MsgReceiver) -> Self {
        Self { receiver: Arc::new(receiver) }
    }

    pub async fn accept(&self) -> Result<LmpChannel> {
        use rustyl4api::vspace::Permission;

        let conn_msg = self.receiver.receive().await?;
        let c_ntf_ep = conn_msg.cap_transfer.ok_or(Error::ProtocolError)?;
        let c_ntf_ep = EpCap::new(c_ntf_ep);

        let receiver = MsgReceiver::new(&EP_SERVER);
        let s_ntf_ep = receiver.badged_ep();
        c_ntf_ep.send(&[], Some(s_ntf_ep.into_slot())).unwrap();

        let shm_msg = receiver.receive().await?;
        let buf_cap = shm_msg.cap_transfer.ok_or(Error::ProtocolError)?;
        let buf_cap = RamCap::new(buf_cap);
        let buf_ptr = gsm!().insert_ram_at(buf_cap, 0, Permission::writable());

        let argbuf = unsafe { ArgumentBuffer::new(buf_ptr as *mut usize, 4096) };
        Ok(LmpChannel::new(
            c_ntf_ep,
            receiver,
            argbuf,
            Role::Server,
        ))
    }

    pub fn incoming(&mut self) -> IncomingFuture {
        IncomingFuture::new(self)
    }

    pub fn derive_connector_ep(&self) -> Option<EpCap> {
        Some(self.receiver.badged_ep())
    }
}

pub struct IncomingFuture<'a> {
    listener: &'a mut LmpListener,
    accept_state: Option<BoxFuture<'a, Result<LmpChannel>>>,
}

impl<'a> IncomingFuture<'a> {
    pub fn new(listener: &'a mut LmpListener) -> Self {
        Self {
            listener,
            accept_state: None,
        }
    }
}

impl<'a> Stream for IncomingFuture<'a> {
    type Item = Result<LmpChannel>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Self {
            listener,
            accept_state,
        } = &mut *self;
        let fut = accept_state.get_or_insert_with(|| {
            let listener = listener.clone();
            let fut = || async move { listener.accept().await };
            Box::pin(fut())
        });

        let chan = ready!(fut.as_mut().poll(cx));

        accept_state.take();
        Poll::Ready(Some(chan))
    }
}
