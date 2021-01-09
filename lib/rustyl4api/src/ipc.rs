pub const IPC_MAX_ARGS: usize = 4;

#[repr(C)]
#[derive(Copy, Clone, Debug, FromPrimitive, ToPrimitive)]
pub enum IpcMessageType {
    Invalid = 0,
    Message,
    Notification,
    Fault,
}

#[derive(Copy, Clone, Debug)]
pub enum IpcMessage {
    Invalid,
    Message {
        payload: [usize; IPC_MAX_ARGS],
        payload_len: usize,
        need_reply: bool,
        cap_transfer: bool,
        badge: Option<usize>,
    },
    Notification(usize),
    Fault,
}
