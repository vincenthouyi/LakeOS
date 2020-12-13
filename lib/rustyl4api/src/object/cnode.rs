use crate::error::SysResult;
use crate::syscall::{MsgInfo, SyscallOp, syscall};

use super::{ObjType, KernelObject, Capability};

pub const CNODE_DEPTH: usize = core::mem::size_of::<usize>() * 8;
pub const CNODE_ENTRY_BIT_SZ: usize = 6;
pub const CNODE_ENTRY_SZ: usize = 1 << CNODE_ENTRY_BIT_SZ;

#[derive(Debug)]
pub enum CNodeObj { }

pub type CNodeCap = Capability<CNodeObj>;

impl KernelObject for CNodeObj {
    fn obj_type() -> ObjType { ObjType::CNode }
}

impl CNodeCap {
    pub fn cap_copy(&self, dst_slot: usize, src_slot: usize) -> SysResult<()> {
        let info = MsgInfo::new(SyscallOp::CapCopy, 2);

        let mut args = [self.slot, dst_slot, src_slot, 0, 0, 0];

        syscall(info, &mut args).map(|_| ())
    }
}