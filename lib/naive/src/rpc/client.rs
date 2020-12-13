use core::{
    task::{Context, Poll},
    pin::Pin,
    future::Future,
};

use alloc::{
    vec::Vec,
    string::String,
};

use rustyl4api::{
    object::{EpCap},
};

use crate::{
    lmp::{LmpChannelHandle, LmpMessage},
};

#[derive(Clone)]
pub struct RpcClient {
    channel: LmpChannelHandle,
}

impl RpcClient {
    fn new(channel: LmpChannelHandle) -> Self {
        Self {
            channel,
        }
    }

    pub fn connect(server_ep: EpCap, ntf_ep: EpCap, ntf_badge: usize) -> Result<Self, ()> {
        let channel = LmpChannelHandle::connect(server_ep, ntf_ep, ntf_badge)?;
        let client = Self::new(channel);
        Ok(client)
    }

    pub fn rpc_write(&mut self, buf: &[u8]) -> RpcCallFuture {
        let payload = super::WriteRequest{ buf: buf.to_vec() };
        let request = LmpMessage {
            opcode: 0,
            msg: serde_json::to_vec(&payload).unwrap(),
            caps: Vec::new(),
        };
        RpcCallFuture::new(self.clone(), request)
    }

    pub fn rpc_read(&mut self, buf: &mut [u8]) -> RpcCallFuture {
        let payload = super::ReadRequest{ len: buf.len() };
        let request = LmpMessage {
            opcode: 1,
            msg: serde_json::to_vec(&payload).unwrap(),
            caps: Vec::new(),
        };
        RpcCallFuture::new(self.clone(), request)
    }

    pub fn request_memory(&mut self, paddr: usize, size: usize, maybe_device: bool) -> RpcCallFuture {
        let payload = super::RequestMemoryRequest{ paddr, size, maybe_device };
        let request = LmpMessage {
            opcode: 2,
            msg: serde_json::to_vec(&payload).unwrap(),
            caps: Vec::new(),
        };
        RpcCallFuture::new(self.clone(), request)
    }

    pub fn request_irq(&mut self, irq: usize) -> RpcCallFuture {
        let payload = super::RequestIrqRequest{ irq };
        let request = LmpMessage {
            opcode: 3,
            msg: serde_json::to_vec(&payload).unwrap(),
            caps: [].to_vec(),
        };
        RpcCallFuture::new(self.clone(), request)
    }

    pub fn register_service(&mut self, name: String, cap: usize) -> RpcCallFuture {
        let payload = super::RegisterServiceRequest { name };
        let request = LmpMessage {
            opcode: 4,
            msg: serde_json::to_vec(&payload).unwrap(),
            caps: [cap].to_vec(),
        };
        RpcCallFuture::new(self.clone(), request)
    }

    pub fn lookup_service(&mut self, name: String) -> RpcCallFuture {
        let payload = super::LookupServiceRequest { name };
        let request = LmpMessage {
            opcode: 5,
            msg: serde_json::to_vec(&payload).unwrap(),
            caps: [].to_vec(),
        };
        RpcCallFuture::new(self.clone(), request)
    }
}

pub struct RpcCallFuture {
    client: RpcClient,
    request: LmpMessage,
    state: usize,
}

impl RpcCallFuture {
    pub fn new(client: RpcClient, request: LmpMessage) -> Self {
        Self { client, request, state: 0 }
    }
}

impl Future for RpcCallFuture {
    type Output = LmpMessage;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self { client, request, state } = &mut *self;

        loop {
            match state {
                0 => {
                    let mut send = client.channel.poll_send(request);
                    let fut = Pin::new(&mut send);
                    ready!(fut.poll(cx)).unwrap();
                    *state = 1;
                }
                1 => {
                    let mut recv = client.channel.poll_recv();
                    let fut = Pin::new(&mut recv);
                    let msg = ready!(fut.poll(cx)).unwrap();
                    return Poll::Ready(msg);
                }
                _ => { unreachable!() }
            }
        }
    }
}
