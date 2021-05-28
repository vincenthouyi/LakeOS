use core::num::NonZeroUsize;

use rustyl4api::error::SysResult;
use rustyl4api::syscall::{syscall, MsgInfo, SyscallOp};

use super::{Capability, KernelObject, ObjType};

pub use rustyl4api::objects::{CNODE_DEPTH, CNODE_ENTRY_BIT_SZ, CNODE_ENTRY_SZ};

#[derive(Debug, Clone)]
pub enum CNodeObj {}

pub type CNodeCap = Capability<CNodeObj>;

impl KernelObject for CNodeObj {
    fn obj_type() -> ObjType {
        ObjType::CNode
    }
}

impl CNodeCap {
    pub fn cap_copy(&self, dst_slot: usize, src_slot: usize) -> SysResult<()> {
        self.cap_copy_badged(dst_slot, src_slot, None)
    }

    pub fn cap_copy_badged(
        &self,
        dst_slot: usize,
        src_slot: usize,
        badge: Option<NonZeroUsize>,
    ) -> SysResult<()> {
        let info = MsgInfo::new(SyscallOp::CapCopy, 3);

        let mut args = [
            self.slot(),
            dst_slot,
            src_slot,
            badge.map(|b| b.get()).unwrap_or(0),
            0,
            0,
        ];

        syscall(info, &mut args).map(|_| ())
    }
}
