use rustyl4api::error::SysResult;
use rustyl4api::syscall::{syscall, MsgInfo, SyscallOp};

use super::{Capability, KernelObject, ObjType};

pub use rustyl4api::objects::{CNODE_DEPTH, CNODE_ENTRY_BIT_SZ, CNODE_ENTRY_SZ};

#[derive(Debug)]
pub enum CNodeObj {}

pub type CNodeCap = Capability<CNodeObj>;

impl KernelObject for CNodeObj {
    fn obj_type() -> ObjType {
        ObjType::CNode
    }
}

impl CNodeCap {
    pub fn cap_copy(&self, dst_slot: usize, src_slot: usize) -> SysResult<()> {
        let info = MsgInfo::new(SyscallOp::CapCopy, 2);

        let mut args = [self.slot, dst_slot, src_slot, 0, 0, 0];

        syscall(info, &mut args).map(|_| ())
    }
}
