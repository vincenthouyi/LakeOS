mod server;
mod client;
mod message;

pub use client::{RpcClient, RpcCallFuture};
pub use server::{RpcServer, RpcRequestHandlers};
pub use message::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Error {
    CallNotSupported,
}

pub type Result<T> = core::result::Result<T, Error>;