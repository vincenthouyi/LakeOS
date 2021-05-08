use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    ops::Drop,
};

use alloc::vec::Vec;

use crate::objects::{EpCap, InterruptCap, RamCap};

use crate::{
    lmp::{LmpChannelHandle, LmpMessage},
    path::{Path, PathBuf},
    Result,
};

use super::message::*;

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

    pub fn connect(server_ep: &EpCap, ntf_ep: EpCap, ntf_badge: usize) -> Result<Self> {
        let channel = LmpChannelHandle::connect(server_ep, ntf_ep, ntf_badge).map_err(|_| crate::Error::Invalid)?;
        let client = Self::new(channel);
        Ok(client)
    }

    pub async fn rpc_write(&mut self, buf: &[u8]) -> Result<usize> {
        let Self { channel, rpc_state } = self;

        let rpc = rpc_state.get_or_insert_with(|| {
            let payload = WriteRequest { buf: buf.to_vec() };
            let payload = RpcRequest {
                opcode: 0,
                payload: serde_json::to_vec(&payload).unwrap(),
            };
            let request = LmpMessage {
                msg: serde_json::to_vec(&payload).unwrap(),
                caps: Vec::new(),
            };
            RpcCallFuture::new(channel.clone(), request)
        });
        let resp_msg = rpc.await;
        let rpc_resp: RpcResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        let rpc_resp = rpc_resp.payload?;
        let resp: WriteResponse = serde_json::from_slice(&rpc_resp).unwrap();
        self.rpc_state.take();
        Ok(resp.result)
    }

    pub async fn rpc_read(&mut self, buf: &mut [u8], offset: usize) -> Result<usize> {
        let Self { channel, rpc_state } = self;

        let rpc = rpc_state.get_or_insert_with(|| {
            let payload = ReadRequest {
                len: buf.len(),
                offset,
            };
            let payload = RpcRequest {
                opcode: 1,
                payload: serde_json::to_vec(&payload).unwrap(),
            };
            let request = LmpMessage {
                msg: serde_json::to_vec(&payload).unwrap(),
                caps: Vec::new(),
            };
            RpcCallFuture::new(channel.clone(), request)
        });
        let resp_msg = rpc.await;
        let rpc_resp: RpcResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        let rpc_resp = rpc_resp.payload?;
        let resp: ReadResponse = serde_json::from_slice(&rpc_resp).unwrap();
        self.rpc_state.take();
        let read_len = buf.len().min(resp.buf.len());
        buf[..read_len].copy_from_slice(&resp.buf[..read_len]);
        Ok(read_len)
    }

    pub async fn request_memory(
        &mut self,
        paddr: usize,
        size: usize,
        maybe_device: bool,
    ) -> Result<RamCap> {
        let Self { channel, rpc_state } = self;

        let rpc = rpc_state.get_or_insert_with(|| {
            let payload = RequestMemoryRequest {
                paddr,
                size,
                maybe_device,
            };
            let payload = RpcRequest {
                opcode: 2,
                payload: serde_json::to_vec(&payload).unwrap(),
            };
            let request = LmpMessage {
                msg: serde_json::to_vec(&payload).unwrap(),
                caps: Vec::new(),
            };
            RpcCallFuture::new(channel.clone(), request)
        });
        let mut resp_msg = rpc.await;
        self.rpc_state.take();
        let rpc_resp: RpcResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        rpc_resp.payload?;
        let cap_slot = resp_msg.caps.pop().unwrap();
        Ok(RamCap::new(cap_slot))
    }

    pub async fn request_irq(&mut self, irq: usize) -> Result<InterruptCap> {
        let Self { channel, rpc_state } = self;

        let rpc = rpc_state.get_or_insert_with(|| {
            let payload = RequestIrqRequest { irq };
            let payload = RpcRequest {
                opcode: 3,
                payload: serde_json::to_vec(&payload).unwrap(),
            };
            let request = LmpMessage {
                msg: serde_json::to_vec(&payload).unwrap(),
                caps: alloc::vec![],
            };
            RpcCallFuture::new(channel.clone(), request)
        });
        let mut resp_msg = rpc.await;
        self.rpc_state.take();
        let rpc_resp: RpcResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        rpc_resp.payload?;
        let cap_slot = resp_msg.caps.pop().unwrap();
        Ok(InterruptCap::new(cap_slot))
    }

    pub async fn register_service<P: AsRef<Path>>(
        &mut self,
        name: P,
        cap: EpCap,
    ) -> Result<()> {
        let Self { channel, rpc_state } = self;

        let rpc = rpc_state.get_or_insert_with(|| {
            let payload = RegisterServiceRequest {
                name: name.as_ref().to_path_buf(),
            };
            let payload = RpcRequest {
                opcode: 4,
                payload: serde_json::to_vec(&payload).unwrap(),
            };
            let request = LmpMessage {
                msg: serde_json::to_vec(&payload).unwrap(),
                caps: alloc::vec![cap.into_slot()],
            };
            RpcCallFuture::new(channel.clone(), request)
        });
        let resp_msg = rpc.await;
        self.rpc_state.take();
        let rpc_resp: RpcResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        rpc_resp.payload.map(|_| ())
    }

    pub async fn lookup_service<P: AsRef<Path>>(&mut self, name: P) -> Result<EpCap> {
        let Self { channel, rpc_state } = self;

        let rpc = rpc_state.get_or_insert_with(|| {
            let payload = LookupServiceRequest {
                name: name.as_ref().to_path_buf(),
            };
            let payload = RpcRequest {
                opcode: 5,
                payload: serde_json::to_vec(&payload).unwrap(),
            };
            let request = LmpMessage {
                msg: serde_json::to_vec(&payload).unwrap(),
                caps: alloc::vec![],
            };
            RpcCallFuture::new(channel.clone(), request)
        });
        let mut resp_msg = rpc.await;
        self.rpc_state.take();
        let rpc_resp: RpcResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        rpc_resp.payload?;
        let cap = resp_msg.caps.pop().unwrap();
        Ok(EpCap::new(cap))
    }

    pub async fn read_dir(&mut self) -> Result<Vec<PathBuf>> {
        let Self { channel, rpc_state } = self;

        let rpc = rpc_state.get_or_insert_with(|| {
            let payload = ReadDirRequest {};
            let payload = RpcRequest {
                opcode: 6,
                payload: serde_json::to_vec(&payload).unwrap(),
            };
            let request = LmpMessage {
                msg: serde_json::to_vec(&payload).unwrap(),
                caps: alloc::vec![],
            };
            RpcCallFuture::new(channel.clone(), request)
        });
        let resp_msg = rpc.await;
        self.rpc_state.take();
        let rpc_resp: RpcResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        let rpc_resp = rpc_resp.payload?;
        let resp: ReadDirResponse = serde_json::from_slice(&rpc_resp).unwrap();
        Ok(resp.filename)
    }
}

impl Drop for RpcClient {
    fn drop(&mut self) {
        self.channel.disconnect();   
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
