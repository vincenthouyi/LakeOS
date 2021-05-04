pub use rustyl4api::ipc::*;
pub use rustyl4api::fault::Fault;

use crate::objects::CapSlot;

#[derive(Debug)]
pub enum IpcMessage {
    Invalid,
    Message(Message),
    Notification(usize),
    Fault(FaultMessage),
}

impl IpcMessage {
    pub fn into_message(self) -> Result<Message, Self> {
        if let IpcMessage::Message(m) = self {
            Ok(m)
        } else {
            Err(self)
        }
    }
}

#[derive(Debug)]
pub struct Message {
    pub payload: [usize; IPC_MAX_ARGS],
    pub payload_len: usize,
    pub need_reply: bool,
    pub cap_transfer: Option<CapSlot>,
    pub badge: Option<usize>,
}

#[derive(Debug)]
pub struct FaultMessage {
    pub badge: Option<usize>,
    pub info: Fault,
}