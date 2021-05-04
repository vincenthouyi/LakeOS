use rustyl4api::error::SysResult;
use rustyl4api::syscall::{syscall, MsgInfo, RespInfo, SyscallOp};
use rustyl4api::fault::Fault;
use crate::objects::ObjType;
use crate::ipc::{self, IpcMessage, IpcMessageType, IPC_MAX_ARGS};

use super::{Capability, KernelObject, CapSlot};

#[derive(Debug, Clone)]
pub struct EndpointObj {}
pub type EpCap = Capability<EndpointObj>;

impl KernelObject for EndpointObj {
    fn obj_type() -> ObjType {
        ObjType::Endpoint
    }
}

impl Capability<EndpointObj> {
    pub fn send(&self, message: &[usize], cap: Option<CapSlot>) -> SysResult<()> {
        let mut args = [self.slot(), 0, 0, 0, 0, 0];
        let len = copy_massge_payload(&mut args, message, &cap);
        let info = MsgInfo::new_ipc(SyscallOp::EndpointSend, len, cap.is_some());

        //TODO handle send fail and return cap
        let ret = syscall(info, &mut args);
        return ret.map(|_| ());
    }

    pub fn receive(&self, cap: Option<CapSlot>) -> SysResult<IpcMessage> {
        let info = MsgInfo::new_ipc(SyscallOp::EndpointRecv, 0, cap.is_some());
        let mut args = [self.slot(), 0, 0, 0, 0, cap.as_ref().map(|c| c.slot()).unwrap_or(0)];
        let (retinfo, retbuf, badge) = syscall(info, &mut args)?;

        handle_receive_return(retinfo, retbuf, badge, cap)
    }

    pub fn reply_receive<'a, 'b>(
        &'a self,
        buf: &'b [usize],
        cap: Option<CapSlot>,
    ) -> SysResult<IpcMessage> {
        let mut args = [self.slot(), 0, 0, 0, 0, 0];
        let has_cap = cap.is_some();
        let len = copy_massge_payload(&mut args, buf, &cap);
        let info = MsgInfo::new_ipc(SyscallOp::EndpointReplyRecv, len, has_cap);

        let (respinfo, retbuf, badge) = syscall(info, &mut args)?;

        handle_receive_return(respinfo, retbuf, badge, cap)
    }

    pub fn call(&self, message: &[usize], cap: Option<CapSlot>) -> SysResult<IpcMessage> {
        let mut args = [self.slot(), 0, 0, 0, 0, 0];
        let has_cap = cap.is_some();
        let len = copy_massge_payload(&mut args, message, &cap);
        let info = MsgInfo::new_ipc(SyscallOp::EndpointCall, len, has_cap);

        let (respinfo, retbuf, badge) = syscall(info, &mut args)?;
        handle_receive_return(respinfo, retbuf, badge, cap)
    }
}

fn handle_receive_return(
    respinfo: RespInfo,
    msgbuf: &[usize],
    badge: usize,
    trans_capslot: Option<CapSlot>,
) -> SysResult<IpcMessage> {
    Ok(match respinfo.msgtype {
        IpcMessageType::Message => {
            let mut real_msgbuf = [0; 4];
            let badge = if respinfo.badged { Some(badge) } else { None };
            let payload_len = msgbuf.len();
            real_msgbuf[..payload_len].copy_from_slice(msgbuf);
            IpcMessage::Message( ipc::Message{
                payload: real_msgbuf,
                payload_len: payload_len,
                need_reply: respinfo.need_reply,
                cap_transfer: respinfo.cap_transfer.then_some(trans_capslot.unwrap()),
                badge: badge,
            })
        }
        IpcMessageType::Fault => {
            let badge = if respinfo.badged { Some(badge) } else { None };
            let fault_info = Fault::from_ipc_message_buf(&msgbuf[0..3]);
            IpcMessage::Fault(ipc::FaultMessage {
                    badge: badge,
                    info: fault_info,
                }
            )
        }
        IpcMessageType::Notification => IpcMessage::Notification(msgbuf[0]),
        IpcMessageType::Invalid => {
            kprintln!(
                "respinfo {:?} msgbuf {:?} badge {}",
                respinfo,
                msgbuf,
                badge
            );
            panic!()
        }
    })
}

pub(crate) fn copy_massge_payload(
    buf: &mut [usize; 6],
    src: &[usize],
    cap_slot: &Option<CapSlot>,
) -> usize {
    let len = src.len().min(IPC_MAX_ARGS);

    buf[1..len + 1].copy_from_slice(&src[..len]);
    buf[5] = cap_slot.as_ref().map(|c| c.slot()).unwrap_or(0);

    len
}
