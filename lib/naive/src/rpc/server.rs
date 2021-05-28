use alloc::{boxed::Box, vec::Vec};

use futures_util::StreamExt;

use crate::lmp::{LmpListener, LmpMessage};
use crate::objects::CapSlot;

use super::message::*;
use crate::{Error, Result};

pub struct RpcServer<T> {
    listener: LmpListener,
    handlers: T,
}

impl<T: RpcRequestHandlers + Sync> RpcServer<T> {
    pub fn new(listener: LmpListener, handlers: T) -> Self {
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
                let mut channel = channel.unwrap();

                loop {
                    let req = channel.poll_recv().await;
                    if let Ok(req) = req {
                        let mut resp = handlers.handle_request(req).await;
                        let res = channel.poll_send(&mut resp).await;
                        if res.is_err() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
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
        Err(Error::NotSupported)
    }

    async fn handle_read(&self, _request: &ReadRequest) -> Result<(ReadResponse, Vec<CapSlot>)> {
        Err(Error::NotSupported)
    }

    async fn handle_request_memory(
        &self,
        _request: &RequestMemoryRequest,
    ) -> Result<(RequestMemoryResponse, Vec<CapSlot>)> {
        Err(Error::NotSupported)
    }

    async fn handle_request_irq(
        &self,
        _request: &RequestIrqRequest,
    ) -> Result<(RequestIrqResponse, Vec<CapSlot>)> {
        Err(Error::NotSupported)
    }

    async fn handle_register_service(
        &self,
        _request: &RegisterServiceRequest,
        _cap: Vec<CapSlot>,
    ) -> Result<(RegisterServiceResponse, Vec<CapSlot>)> {
        Err(Error::NotSupported)
    }

    async fn handle_lookup_service(
        &self,
        _request: &LookupServiceRequest,
    ) -> Result<(LookupServiceResponse, Vec<CapSlot>)> {
        Err(Error::NotSupported)
    }

    async fn handle_read_dir(
        &self,
        _request: &ReadDirRequest,
    ) -> Result<(ReadDirResponse, Vec<CapSlot>)> {
        Err(Error::NotSupported)
    }

    async fn __handle_request(
        &self,
        opcode: u8,
        request: RpcRequest,
        cap: Vec<CapSlot>,
    ) -> Result<(Vec<u8>, Vec<CapSlot>)> {
        let r = match opcode {
            0 => {
                let request: WriteRequest =
                    serde_json::from_slice(&request.payload).map_err(|_| Error::Invalid)?;
                let (resp, cap) = self.handle_write(&request).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            1 => {
                let request: ReadRequest =
                    serde_json::from_slice(&request.payload).map_err(|_| Error::Invalid)?;
                let (resp, cap) = self.handle_read(&request).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            2 => {
                let request =
                    serde_json::from_slice(&request.payload).map_err(|_| Error::Invalid)?;
                let (resp, cap) = self.handle_request_memory(&request).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            3 => {
                let request =
                    serde_json::from_slice(&request.payload).map_err(|_| Error::Invalid)?;
                let (resp, cap) = self.handle_request_irq(&request).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            4 => {
                let request_msg =
                    serde_json::from_slice(&request.payload).map_err(|_| Error::Invalid)?;
                let (resp, cap) = self.handle_register_service(&request_msg, cap).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            5 => {
                let request_msg: LookupServiceRequest =
                    serde_json::from_slice(&request.payload).map_err(|_| Error::Invalid)?;
                let (resp, cap) = self.handle_lookup_service(&request_msg).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            6 => {
                let request_msg =
                    serde_json::from_slice(&request.payload).map_err(|_| Error::Invalid)?;
                let (resp, cap) = self.handle_read_dir(&request_msg).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            _ => {
                todo!()
            }
        };
        Ok(r)
    }

    async fn handle_request(&self, msg: LmpMessage) -> LmpMessage {
        let (rpc_resp, caps) = if let Ok(request) = serde_json::from_slice::<RpcRequest>(&msg.msg) {
            let opcode = request.opcode;
            self.__handle_request(opcode, request, msg.caps)
                .await
                .map_or_else(
                    |e| (RpcResponse { payload: Err(e) }, Vec::new()),
                    |(rpc_resp, caps)| {
                        (
                            RpcResponse {
                                payload: Ok(rpc_resp),
                            },
                            caps,
                        )
                    },
                )
        } else {
            (
                RpcResponse {
                    payload: Err(Error::Invalid),
                },
                Vec::new(),
            )
        };
        LmpMessage {
            msg: serde_json::to_vec(&rpc_resp).unwrap(),
            caps,
        }
    }
}
