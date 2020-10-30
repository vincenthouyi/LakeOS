use crate::object::ObjType;
use crate::error::SysResult;
use crate::syscall::{MsgInfo, RespInfo, SyscallOp, syscall};
use crate::ipc::{IpcMessage, IpcMessageType, IPC_MAX_ARGS};

use super::{Capability, KernelObject};

#[derive(Debug, Clone)]
pub struct EndpointObj {}
pub type EpCap = Capability<EndpointObj>;

impl KernelObject for EndpointObj {
    fn obj_type() -> ObjType { ObjType::Endpoint }
}

impl Capability<EndpointObj> {
    pub fn mint(&self, dst_slot: usize, badge: usize) -> SysResult<()> {
        let mut args = [self.slot, dst_slot, badge, 0, 0, 0];
        let info = MsgInfo::new(SyscallOp::EndpointMint, 2);

        let ret = syscall(info, &mut args);
        return ret.map(|_|());
    }

    pub fn send(&self, message: &[usize], cap: Option<usize>) -> SysResult<()> {
        let mut args = [self.slot, 0, 0, 0, 0, 0];
        let len = copy_massge_payload(&mut args, message, cap);
        let info = MsgInfo::new_ipc(SyscallOp::EndpointSend, len, cap.is_some());

        let ret = syscall(info, &mut args);
        return ret.map(|_|());
    }

    pub fn receive(&self, cap: Option<usize>) -> SysResult<IpcMessage> {
        let info = MsgInfo::new_ipc(SyscallOp::EndpointRecv, 0, cap.is_some());
        let mut args = [self.slot, 0, 0, 0, 0, cap.unwrap_or(0)];
        let (retinfo, retbuf, badge) = syscall(info, &mut args)?;

        handle_receive_return(retinfo, retbuf, badge)
    }

    pub fn reply_receive<'a, 'b>(&'a self, buf: &'b [usize], cap: Option<usize>)
        -> SysResult<IpcMessage>
    {
        let mut args = [self.slot, 0, 0, 0, 0, 0];
        let len = copy_massge_payload(&mut args, buf, cap);
        let info = MsgInfo::new_ipc(SyscallOp::EndpointReplyRecv, len, cap.is_some());

        let (respinfo, retbuf, badge) = syscall(info, &mut args)?;

        handle_receive_return(respinfo, retbuf, badge)
    }

    pub fn call(&self, message: &[usize], cap: Option<usize>) -> SysResult<IpcMessage> {
        let mut args = [self.slot, 0, 0, 0, 0, cap.unwrap_or(0)];
        let len = copy_massge_payload(&mut args, message, cap);
        let info = MsgInfo::new_ipc(SyscallOp::EndpointCall, len, cap.is_some());

        let (resp_info, retbuf, _) = syscall(info, &mut args)?;
        let mut real_retbuf = [0; 4];
        let payload_len = retbuf.len();
        real_retbuf[..payload_len].copy_from_slice(retbuf);

        Ok(IpcMessage::Message {
            payload: real_retbuf,
            payload_len: payload_len,
            need_reply: resp_info.need_reply,
            cap_transfer: resp_info.cap_transfer,
            badge: None,
        })
    }
}

fn handle_receive_return(respinfo: RespInfo, msgbuf: &[usize], badge: usize)
    -> SysResult<IpcMessage>
{
    Ok(match respinfo.msgtype {
        IpcMessageType::Message => {
            let mut real_msgbuf = [0; 4];
            let badge = if respinfo.badged {
                Some(badge)
            } else {
                None
            };
            let payload_len = msgbuf.len();
            real_msgbuf[..payload_len].copy_from_slice(msgbuf);
            IpcMessage::Message {
                payload: real_msgbuf,
                payload_len: payload_len,
                need_reply: respinfo.need_reply,
                cap_transfer: respinfo.cap_transfer,
                badge: badge
            }
        }
        IpcMessageType::Fault => {
            unimplemented!()
        }
        IpcMessageType::Notification => {
            IpcMessage::Notification(msgbuf[0])
        }
        IpcMessageType::Invalid => {
            // FIXME: find why panic without underlying kprintln
            kprintln!("respinfo {:?} msgbuf {:?} badge {}", respinfo, msgbuf, badge);
            panic!()
        }
    })
}

pub(crate) fn copy_massge_payload(buf: &mut [usize;6], src: &[usize], cap_slot: Option<usize>) -> usize {
    let len = src.len().min(IPC_MAX_ARGS);

    buf[1..len + 1].copy_from_slice(&src[..len]);
    buf[5] = cap_slot.unwrap_or(0);

    len
}