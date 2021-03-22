use alloc::{boxed::Box, vec::Vec};

use futures_util::StreamExt;

use crate::lmp::{LmpChannelHandle, LmpListenerHandle, LmpMessage};
use crate::objects::CapSlot;

use super::message::*;
use super::{Error, Result};

pub struct RpcServer<T> {
    listener: LmpListenerHandle,
    handlers: T,
}

impl<T: RpcRequestHandlers + Sync> RpcServer<T> {
    pub fn new(listener: LmpListenerHandle, handlers: T) -> Self {
        Self {
            listener: listener,
            handlers: handlers,
        }
    }

    pub async fn run(self) {
        let Self {
            mut listener,
            handlers,
        } = self;

        listener
            .incoming()
            .for_each_concurrent(None, |channel| async {
                let channel = channel.unwrap();

                channel
                    .messages()
                    .for_each_concurrent(None, |req| async {
                        handlers.handle_request(channel.clone(), req).await;
                    })
                    .await;
            })
            .await;
    }

    // pub fn derive_connector_ep(&self) -> Option<EpCap> {
    //     self.listener.derive_connector_ep()
    // }
}

#[async_trait]
pub trait RpcRequestHandlers {
    async fn handle_write(&self, _request: &WriteRequest) -> Result<(WriteResponse, Vec<CapSlot>)> {
        Err(Error::CallNotSupported)
    }

    async fn handle_read(&self, _request: &ReadRequest) -> Result<(ReadResponse, Vec<CapSlot>)> {
        Err(Error::CallNotSupported)
    }

    async fn handle_request_memory(
        &self,
        _request: &RequestMemoryRequest,
    ) -> Result<(RequestMemoryResponse, Vec<CapSlot>)> {
        Err(Error::CallNotSupported)
    }

    async fn handle_request_irq(
        &self,
        _request: &RequestIrqRequest,
    ) -> Result<(RequestIrqResponse, Vec<CapSlot>)> {
        Err(Error::CallNotSupported)
    }

    async fn handle_register_service(
        &self,
        _request: &RegisterServiceRequest,
        _cap: Vec<CapSlot>,
    ) -> Result<(RegisterServiceResponse, Vec<CapSlot>)> {
        Err(Error::CallNotSupported)
    }

    async fn handle_lookup_service(
        &self,
        _request: &LookupServiceRequest,
    ) -> Result<(LookupServiceResponse, Vec<CapSlot>)> {
        Err(Error::CallNotSupported)
    }

    async fn handle_read_dir(
        &self,
        _request: &ReadDirRequest,
    ) -> Result<(ReadDirResponse, Vec<CapSlot>)> {
        Err(Error::CallNotSupported)
    }

    async fn _handle_request(&self, request: LmpMessage) -> Result<(Vec<u8>, Vec<CapSlot>)> {
        let opcode = request.opcode;
        let r = match opcode {
            0 => {
                let request: WriteRequest =
                    serde_json::from_slice(&request.msg).map_err(|_| Error::BadRequest)?;
                let (resp, cap) = self.handle_write(&request).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            1 => {
                let request: ReadRequest =
                    serde_json::from_slice(&request.msg).map_err(|_| Error::BadRequest)?;
                let (resp, cap) = self.handle_read(&request).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            2 => {
                let request =
                    serde_json::from_slice(&request.msg).map_err(|_| Error::BadRequest)?;
                let (resp, cap) = self.handle_request_memory(&request).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            3 => {
                let request =
                    serde_json::from_slice(&request.msg).map_err(|_| Error::BadRequest)?;
                let (resp, cap) = self.handle_request_irq(&request).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            4 => {
                let request_msg =
                    serde_json::from_slice(&request.msg).map_err(|_| Error::BadRequest)?;
                let (resp, cap) = self
                    .handle_register_service(&request_msg, request.caps)
                    .await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            5 => {
                let request_msg =
                    serde_json::from_slice(&request.msg).map_err(|_| Error::BadRequest)?;
                let (resp, cap) = self.handle_lookup_service(&request_msg).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            6 => {
                let request_msg =
                    serde_json::from_slice(&request.msg).map_err(|_| Error::BadRequest)?;
                let (resp, cap) = self.handle_read_dir(&request_msg).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            _ => {
                todo!()
            }
        };
        Ok(r)
    }

    async fn handle_request(&self, channel: LmpChannelHandle, request: LmpMessage) {
        let opcode = request.opcode;
        let (resp_payload, cap) = self._handle_request(request).await.unwrap();
        let mut resp = LmpMessage {
            opcode: opcode,
            msg: resp_payload,
            caps: cap,
        };
        channel.poll_send(&mut resp).await.unwrap();
    }
}
