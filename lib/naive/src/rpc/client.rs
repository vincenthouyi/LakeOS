use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use alloc::{vec::Vec};

use rustyl4api::object::EpCap;

use crate::{
    lmp::{LmpChannelHandle, LmpMessage},
    ns,
    path::{Path, PathBuf},
};

pub struct RpcClient {
    channel: LmpChannelHandle,
    rpc_state: Option<RpcCallFuture>,
}

impl RpcClient {
    fn new(channel: LmpChannelHandle) -> Self {
        Self {
            channel,
            rpc_state: None,
        }
    }

    pub fn connect(server_ep: EpCap, ntf_ep: EpCap, ntf_badge: usize) -> Result<Self, ()> {
        let channel = LmpChannelHandle::connect(server_ep, ntf_ep, ntf_badge)?;
        let client = Self::new(channel);
        Ok(client)
    }

    pub async fn rpc_write(&mut self, buf: &[u8]) -> usize {
        let Self { channel, rpc_state } = self;

        let rpc = rpc_state.get_or_insert_with(|| {
            let payload = super::WriteRequest { buf: buf.to_vec() };
            let request = LmpMessage {
                opcode: 0,
                msg: serde_json::to_vec(&payload).unwrap(),
                caps: Vec::new(),
            };
            RpcCallFuture::new(channel.clone(), request)
        });
        let resp_msg = rpc.await;
        let resp: super::WriteResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        self.rpc_state.take();
        resp.result
    }

    pub async fn rpc_read(&mut self, buf: &mut [u8], offset: usize) -> usize {
        let Self { channel, rpc_state } = self;

        let rpc = rpc_state.get_or_insert_with(|| {
            let payload = super::ReadRequest { len: buf.len(), offset };
            let request = LmpMessage {
                opcode: 1,
                msg: serde_json::to_vec(&payload).unwrap(),
                caps: Vec::new(),
            };
            RpcCallFuture::new(channel.clone(), request)
        });
        let resp_msg = rpc.await;
        let resp: super::ReadResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        self.rpc_state.take();
        let read_len = buf.len().min(resp.buf.len());
        buf[..read_len].copy_from_slice(&resp.buf[..read_len]);
        read_len
    }

    pub async fn request_memory(
        &mut self,
        paddr: usize,
        size: usize,
        maybe_device: bool,
    ) -> Result<usize, ()> {
        let Self { channel, rpc_state } = self;

        let rpc = rpc_state.get_or_insert_with(|| {
            let payload = super::RequestMemoryRequest {
                paddr,
                size,
                maybe_device,
            };
            let request = LmpMessage {
                opcode: 2,
                msg: serde_json::to_vec(&payload).unwrap(),
                caps: Vec::new(),
            };
            RpcCallFuture::new(channel.clone(), request)
        });
        let resp_msg = rpc.await;
        let resp: super::RequestMemoryResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        self.rpc_state.take();
        match resp.result {
            0 => Ok(*resp_msg.caps.get(0).unwrap()),
            _ => Err(()),
        }
    }

    pub async fn request_irq(&mut self, irq: usize) -> Result<usize, ()> {
        let Self { channel, rpc_state } = self;

        let rpc = rpc_state.get_or_insert_with(|| {
            let payload = super::RequestIrqRequest { irq };
            let request = LmpMessage {
                opcode: 3,
                msg: serde_json::to_vec(&payload).unwrap(),
                caps: [].to_vec(),
            };
            RpcCallFuture::new(channel.clone(), request)
        });
        let resp_msg = rpc.await;
        let resp: super::RequestIrqResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        self.rpc_state.take();
        match resp.result {
            0 => Ok(*resp_msg.caps.get(0).unwrap()),
            _ => Err(()),
        }
    }

    pub async fn register_service<P: AsRef<Path>>(&mut self, name: P, cap: usize) -> ns::Result<()> {
        let Self { channel, rpc_state } = self;

        let rpc = rpc_state.get_or_insert_with(|| {
            let payload = super::RegisterServiceRequest { name: name.as_ref().to_path_buf() };
            let request = LmpMessage {
                opcode: 4,
                msg: serde_json::to_vec(&payload).unwrap(),
                caps: [cap].to_vec(),
            };
            RpcCallFuture::new(channel.clone(), request)
        });
        let resp_msg = rpc.await;
        let resp: super::RegisterServiceResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        self.rpc_state.take();
        resp.result.into_result()
    }

    pub async fn lookup_service<P: AsRef<Path>>(&mut self, name: P) -> ns::Result<usize> {
        let Self { channel, rpc_state } = self;

        let rpc = rpc_state.get_or_insert_with(|| {
            let payload = super::LookupServiceRequest { name: name.as_ref().to_path_buf() };
            let request = LmpMessage {
                opcode: 5,
                msg: serde_json::to_vec(&payload).unwrap(),
                caps: [].to_vec(),
            };
            RpcCallFuture::new(channel.clone(), request)
        });
        let resp_msg = rpc.await;
        let resp: super::LookupServiceResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        self.rpc_state.take();
        resp.result
            .into_result()
            .map(|_| *resp_msg.caps.get(0).unwrap())
    }

    pub async fn read_dir(&mut self) -> ns::Result<Vec<PathBuf>> {
        let Self { channel, rpc_state } = self;

        let rpc = rpc_state.get_or_insert_with(|| {
            let payload = super::ReadDirRequest {};
            let request = LmpMessage {
                opcode: 6,
                msg: serde_json::to_vec(&payload).unwrap(),
                caps: [].to_vec(),
            };
            RpcCallFuture::new(channel.clone(), request)
        });
        let resp_msg = rpc.await;
        let resp: super::ReadDirResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        self.rpc_state.take();
        Ok(resp.filename)
    }
}

pub struct RpcCallFuture {
    channel: LmpChannelHandle,
    request: LmpMessage,
    state: usize,
}

impl RpcCallFuture {
    pub fn new(channel: LmpChannelHandle, request: LmpMessage) -> Self {
        Self {
            channel,
            request,
            state: 0,
        }
    }
}

impl Future for RpcCallFuture {
    type Output = LmpMessage;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self {
            channel,
            request,
            state,
        } = &mut *self;

        loop {
            match state {
                0 => {
                    let mut send = channel.poll_send(request);
                    let fut = Pin::new(&mut send);
                    ready!(fut.poll(cx)).unwrap();
                    *state = 1;
                }
                1 => {
                    let mut recv = channel.poll_recv();
                    let fut = Pin::new(&mut recv);
                    let msg = ready!(fut.poll(cx)).unwrap();
                    return Poll::Ready(msg);
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }
}
