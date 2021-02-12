use alloc::vec::Vec;

use crate::ns;
use crate::path::PathBuf;

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
pub struct RequestMemoryResponse {
    pub result: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestIrqRequest {
    pub irq: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestIrqResponse {
    pub result: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterServiceRequest {
    pub name: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterServiceResponse {
    pub result: ns::Error,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LookupServiceRequest {
    pub name: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LookupServiceResponse {
    pub result: ns::Error,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadDirRequest {
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadDirResponse {
    pub filename: Vec<PathBuf>,
}
