pub const IPC_MAX_ARGS: usize = 4;

#[repr(C)]
#[derive(Copy, Clone, Debug, FromPrimitive, ToPrimitive)]
pub enum IpcMessageType {
    Invalid = 0,
    Message,
    Notification,
    Fault,
}
