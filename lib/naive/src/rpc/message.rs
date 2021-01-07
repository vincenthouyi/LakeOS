use alloc::vec::Vec;
use alloc::string::String;
use crate::ns;

#[derive(Serialize, Deserialize, Debug)]
pub struct WriteRequest {
    pub buf: Vec<u8>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WriteResponse {
    pub result: usize
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadRequest {
    pub len: usize
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadResponse {
    pub buf: Vec<u8>
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
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterServiceResponse {
    pub result: ns::Error,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LookupServiceRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LookupServiceResponse {
    pub result: ns::Error,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CurrentTimeRequest {
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CurrentTimeResponse {
    pub time: u64,
}