use rustyl4api::error::SysResult;
use rustyl4api::syscall::{syscall, MsgInfo, SyscallOp};

use super::{Capability, KernelObject, ObjType};

#[derive(Debug, Clone)]
pub struct VTableObj {}

pub type VTableCap = Capability<VTableObj>;

impl KernelObject for VTableObj {
    fn obj_type() -> ObjType {
        ObjType::VTable
    }
}

impl Capability<VTableObj> {
    pub fn map(&self, vspace: &VTableCap, vaddr: usize, level: usize) -> SysResult<()> {
        let info = MsgInfo::new(SyscallOp::VTableMap, 3);
        let mut args = [self.slot(), vspace.slot(), vaddr, level, 0, 0];
        syscall(info, &mut args).map(|_| ())
    }

    pub fn unmap(&self) -> SysResult<()> {
        unimplemented!()
    }
}
