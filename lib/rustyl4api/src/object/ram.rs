use crate::error::SysResult;
use crate::object::ObjType;
use crate::syscall::{syscall, MsgInfo, SyscallOp};
use crate::vspace::Permission;

use super::{Capability, KernelObject};

#[derive(Debug, Clone)]
pub struct RamObj {}

pub type RamCap = Capability<RamObj>;

impl KernelObject for RamObj {
    fn obj_type() -> ObjType {
        ObjType::Ram
    }
}

impl Capability<RamObj> {
    pub fn map(&self, vspace: usize, vaddr: usize, rights: Permission) -> SysResult<()> {
        let info = MsgInfo::new(SyscallOp::RamMap, 3);
        let mut args = [self.slot, vspace, vaddr, rights.into(), 0, 0];
        syscall(info, &mut args).map(|_| ())
    }

    pub fn unmap(&self) -> SysResult<()> {
        unimplemented!()
    }

    pub fn derive(&self, dst_cptr: usize) -> SysResult<()> {
        let info = MsgInfo::new(SyscallOp::Derive, 1);
        let mut args = [self.slot, dst_cptr, 0, 0, 0, 0];
        syscall(info, &mut args).map(|_| ())
    }
}
