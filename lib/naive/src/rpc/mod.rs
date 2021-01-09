mod client;
mod message;
mod server;

pub use client::{RpcCallFuture, RpcClient};
pub use message::*;
pub use server::{RpcRequestHandlers, RpcServer};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Error {
    CallNotSupported,
}

pub type Result<T> = core::result::Result<T, Error>;
