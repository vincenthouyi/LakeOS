pub use rustyl4api::ipc::*;

use crate::objects::CapSlot;

#[derive(Debug)]
pub enum IpcMessage {
    Invalid,
    Message(Message),
    Notification(usize),
    Fault,
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