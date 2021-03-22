use rustyl4api::error::SysResult;
use rustyl4api::syscall::{syscall, MsgInfo, SyscallOp};
use rustyl4api::vspace::Permission;
use crate::objects::ObjType;

use super::{Capability, KernelObject, VTableCap};

#[derive(Debug, Clone)]
pub struct RamObj {}

pub type RamCap = Capability<RamObj>;

impl KernelObject for RamObj {
    fn obj_type() -> ObjType {
        ObjType::Ram
    }
}

impl Capability<RamObj> {
    pub fn map(&self, vspace: &VTableCap, vaddr: usize, rights: Permission) -> SysResult<()> {
        let info = MsgInfo::new(SyscallOp::RamMap, 3);
        let mut args = [self.slot(), vspace.slot(), vaddr, rights.into(), 0, 0];
        syscall(info, &mut args).map(|_| ())
    }

    pub fn unmap(&self) -> SysResult<()> {
        unimplemented!()
    }
}
