use core::future::Future;
use alloc::{boxed::Box, vec::Vec};
use core::pin::Pin;

use futures_util::StreamExt;

use crate::lmp::{LmpListener, LmpMessage};
use crate::objects::CapSlot;

use super::message::*;
use crate::rpc::Service;
use crate::{Error, Result};

pub struct RpcServer {
    listener: LmpListener,
}

impl RpcServer {
    pub fn new(listener: LmpListener) -> Self {
        Self { listener }
    }

    pub async fn run<T>(&mut self, handler: T)
    where
        T: Service<LmpMessage, Response = LmpMessage> + Clone
    {
        self.listener
            .incoming()
            .for_each_concurrent(None, |channel| async {
                let mut channel = channel.unwrap();
                let mut handler = handler.clone();

                loop {
                    let req = channel.poll_recv().await;
                    if let Ok(req) = req {
                        let resp = handler.call(req).await;
                        if let Ok(mut r) = resp {
                            let res = channel.poll_send(&mut r).await;
                            if res.is_err() {
                                break;
                            }
                        } else {
                            unimplemented!()
                        }
                    } else {
                        break;
                    }
                }
            })
            .await;
    }
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
}

#[derive(Clone)]
pub struct RpcServerHandler<T: Clone> {
    handler: T,
}

impl<T: RpcRequestHandlers + Sync + Clone> RpcServerHandler<T> {
    pub fn new(handler: T) -> Self {
        Self { handler }
    }

    async fn handle_request(
        &self,
        opcode: u8,
        request: RpcRequest,
        cap: Vec<CapSlot>,
    ) -> Result<(Vec<u8>, Vec<CapSlot>)> {
        let r = match opcode {
            0 => {
                let request: WriteRequest =
                    serde_json::from_slice(&request.payload).map_err(|_| Error::Invalid)?;
                let (resp, cap) = self.handler.handle_write(&request).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            1 => {
                let request: ReadRequest =
                    serde_json::from_slice(&request.payload).map_err(|_| Error::Invalid)?;
                let (resp, cap) = self.handler.handle_read(&request).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            2 => {
                let request =
                    serde_json::from_slice(&request.payload).map_err(|_| Error::Invalid)?;
                let (resp, cap) = self.handler.handle_request_memory(&request).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            3 => {
                let request =
                    serde_json::from_slice(&request.payload).map_err(|_| Error::Invalid)?;
                let (resp, cap) = self.handler.handle_request_irq(&request).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            4 => {
                let request_msg =
                    serde_json::from_slice(&request.payload).map_err(|_| Error::Invalid)?;
                let (resp, cap) = self.handler.handle_register_service(&request_msg, cap).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            5 => {
                let request_msg: LookupServiceRequest =
                    serde_json::from_slice(&request.payload).map_err(|_| Error::Invalid)?;
                let (resp, cap) = self.handler.handle_lookup_service(&request_msg).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            6 => {
                let request_msg =
                    serde_json::from_slice(&request.payload).map_err(|_| Error::Invalid)?;
                let (resp, cap) = self.handler.handle_read_dir(&request_msg).await?;
                (serde_json::to_vec(&resp).unwrap(), cap)
            }
            _ => {
                todo!()
            }
        };
        Ok(r)
    }
}

impl<T: 'static + RpcRequestHandlers + Sync + Clone> Service<LmpMessage> for RpcServerHandler<T> {
    type Response = LmpMessage;

    type Error = Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response>>>>;

    fn call(&mut self, msg: LmpMessage) -> Self::Future {
        let this = self.clone();
        let fut = async move {
            let (rpc_resp, caps) = if let Ok(request) = serde_json::from_slice::<RpcRequest>(&msg.msg) {
                let opcode = request.opcode;
                this.handle_request(opcode, request, msg.caps)
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
            Ok(LmpMessage {
                msg: serde_json::to_vec(&rpc_resp).unwrap(),
                caps,
            })
        };
        Box::pin(fut)
    }
}
