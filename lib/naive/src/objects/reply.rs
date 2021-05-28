use crate::objects::{CapSlot, ObjType};
use rustyl4api::error::SysResult;
use rustyl4api::syscall::{syscall, MsgInfo, SyscallOp};

use super::{Capability, KernelObject};

#[derive(Debug, Clone)]
pub struct ReplyObj {}
pub type ReplyCap = Capability<ReplyObj>;

impl KernelObject for ReplyObj {
    fn obj_type() -> ObjType {
        ObjType::Reply
    }
}

impl Capability<ReplyObj> {
    pub fn reply(&self, message: &[usize], cap: Option<CapSlot>) -> SysResult<()> {
        let mut args = [self.slot(), 0, 0, 0, 0, 0];
        let len = super::endpoint::copy_massge_payload(&mut args, message, &cap);
        let info = MsgInfo::new_ipc(SyscallOp::EndpointReply, len, cap.is_some());
        let ret = syscall(info, &mut args);
        return ret.map(|_| ());
    }
}
