use core::{
    pin::Pin,
    task::{Context, Poll},
};

use alloc::boxed::Box;

use futures_util::future::BoxFuture;
use futures_util::ready;
use futures_util::stream::Stream;

use crate::{
    ep_receiver::EpReceiver,
    ep_server::EP_SERVER,
    objects::{EpCap, RamCap},
    space_manager::{copy_cap, gsm},
    Result,
};

use super::{ArgumentBuffer, LmpChannel, LmpChannelHandle, Role};

#[derive(Clone)]
pub struct LmpListener {
    receiver: EpReceiver,
}

impl LmpListener {
    pub fn new(receiver: EpReceiver) -> Self {
        Self { receiver }
    }

    pub async fn accept(&self) -> Result<LmpChannelHandle> {
        use rustyl4api::vspace::Permission;

        let conn_msg = self.receiver.receive().await?;
        let c_ntf_ep = EpCap::new(conn_msg.cap_transfer.unwrap());

        let receiver = EP_SERVER.derive_receiver();
        let s_ntf_ep = copy_cap(&receiver.ep).unwrap();
        c_ntf_ep.send(&[], Some(s_ntf_ep.into_slot())).unwrap();

        let shm_msg = receiver.receive().await?;
        let buf_cap = RamCap::new(shm_msg.cap_transfer.unwrap());
        let buf_ptr = gsm!().insert_ram_at(buf_cap, 0, Permission::writable());

        let argbuf = unsafe { ArgumentBuffer::new(buf_ptr as *mut usize, 4096) };
        Ok(LmpChannelHandle::from_inner(LmpChannel::new(
            c_ntf_ep,
            receiver,
            argbuf,
            Role::Server,
        )))
    }

    pub fn incoming(&mut self) -> IncomingFuture {
        IncomingFuture::new(self)
    }

    pub fn derive_connector_ep(&self) -> Option<EpCap> {
        copy_cap(&self.receiver.ep)
    }
}

pub struct IncomingFuture<'a> {
    listener: &'a mut LmpListener,
    accept_state: Option<BoxFuture<'a, Result<LmpChannelHandle>>>,
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
    type Item = Result<LmpChannelHandle>;

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
