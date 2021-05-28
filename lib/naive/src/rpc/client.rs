use alloc::vec::Vec;

use crate::objects::{EpCap, InterruptCap, RamCap};

use crate::{
    ep_receiver::EpReceiver,
    lmp::{LmpChannel, LmpMessage},
    path::{Path, PathBuf},
    Result,
};

use super::message::*;

pub struct RpcClient {
    channel: LmpChannel,
}

impl RpcClient {
    fn new(channel: LmpChannel) -> Self {
        Self { channel }
    }

    pub async fn connect(server_ep: &EpCap, receiver: EpReceiver) -> Result<Self> {
        let channel = LmpChannel::connect(server_ep, receiver)
            .await
            .map_err(|_| crate::Error::Invalid)?;
        let client = Self::new(channel);
        Ok(client)
    }

    async fn rpc_call(&mut self, mut request: LmpMessage) -> Result<LmpMessage> {
        self.channel.poll_send(&mut request).await?;
        self.channel.poll_recv().await
    }

    pub async fn rpc_write(&mut self, buf: Vec<u8>) -> Result<usize> {
        let payload = WriteRequest { buf: buf };
        let payload = RpcRequest {
            opcode: 0,
            payload: serde_json::to_vec(&payload).unwrap(),
        };
        let request = LmpMessage {
            msg: serde_json::to_vec(&payload).unwrap(),
            caps: Vec::new(),
        };

        let resp_msg = self.rpc_call(request).await?;
        let rpc_resp: RpcResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        let rpc_resp = rpc_resp.payload?;
        let resp: WriteResponse = serde_json::from_slice(&rpc_resp).unwrap();
        Ok(resp.result)
    }

    pub async fn rpc_read(&mut self, buflen: usize, offset: usize) -> Result<Vec<u8>> {
        let payload = ReadRequest {
            len: buflen,
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
        let resp_msg = self.rpc_call(request).await?;
        let rpc_resp: RpcResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        let rpc_resp = rpc_resp.payload?;
        let resp: ReadResponse = serde_json::from_slice(&rpc_resp).unwrap();
        Ok(resp.buf)
    }

    pub async fn request_memory(
        &mut self,
        paddr: usize,
        size: usize,
        maybe_device: bool,
    ) -> Result<RamCap> {
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
        let mut resp_msg = self.rpc_call(request).await?;
        let rpc_resp: RpcResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        rpc_resp.payload?;
        let cap_slot = resp_msg.caps.pop().unwrap();
        Ok(RamCap::new(cap_slot))
    }

    pub async fn request_irq(&mut self, irq: usize) -> Result<InterruptCap> {
        let payload = RequestIrqRequest { irq };
        let payload = RpcRequest {
            opcode: 3,
            payload: serde_json::to_vec(&payload).unwrap(),
        };
        let request = LmpMessage {
            msg: serde_json::to_vec(&payload).unwrap(),
            caps: alloc::vec![],
        };
        let mut resp_msg = self.rpc_call(request).await?;
        let rpc_resp: RpcResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        rpc_resp.payload?;
        let cap_slot = resp_msg.caps.pop().unwrap();
        Ok(InterruptCap::new(cap_slot))
    }

    pub async fn register_service<P: AsRef<Path>>(&mut self, name: P, cap: EpCap) -> Result<()> {
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
        let resp_msg = self.rpc_call(request).await?;
        let rpc_resp: RpcResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        rpc_resp.payload.map(|_| ())
    }

    pub async fn lookup_service<P: AsRef<Path>>(&mut self, name: P) -> Result<EpCap> {
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
        let mut resp_msg = self.rpc_call(request).await?;
        let rpc_resp: RpcResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        rpc_resp.payload?;
        let cap = resp_msg.caps.pop().unwrap();
        Ok(EpCap::new(cap))
    }

    pub async fn read_dir(&mut self) -> Result<Vec<PathBuf>> {
        let payload = ReadDirRequest {};
        let payload = RpcRequest {
            opcode: 6,
            payload: serde_json::to_vec(&payload).unwrap(),
        };
        let request = LmpMessage {
            msg: serde_json::to_vec(&payload).unwrap(),
            caps: alloc::vec![],
        };
        let resp_msg = self.rpc_call(request).await?;
        let rpc_resp: RpcResponse = serde_json::from_slice(&resp_msg.msg).unwrap();
        let rpc_resp = rpc_resp.payload?;
        let resp: ReadDirResponse = serde_json::from_slice(&rpc_resp).unwrap();
        Ok(resp.filename)
    }
}
