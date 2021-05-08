use alloc::vec::Vec;

use serde::{Serialize, Deserialize};

use crate::path::PathBuf;

use crate::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcRequest {
    pub opcode: u8,
    pub payload: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcResponse {
    pub payload: Result<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WriteRequest {
    pub buf: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WriteResponse {
    pub result: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadRequest {
    pub len: usize,
    pub offset: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadResponse {
    pub buf: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestMemoryRequest {
    pub paddr: usize,
    pub size: usize,
    pub maybe_device: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestMemoryResponse {}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestIrqRequest {
    pub irq: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestIrqResponse {}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterServiceRequest {
    pub name: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterServiceResponse {}

#[derive(Serialize, Deserialize, Debug)]
pub struct LookupServiceRequest {
    pub name: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LookupServiceResponse {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadDirRequest {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadDirResponse {
    pub filename: Vec<PathBuf>,
}
