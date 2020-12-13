use crate::object::{ObjType};
use crate::error::SysResult;
use crate::syscall::{MsgInfo, SyscallOp, syscall};

use super::{Capability, KernelObject};

#[derive(Debug)]
pub struct TcbObj {}
pub type TcbCap = Capability<TcbObj>;

pub const TCB_OBJ_SZ: usize = 1024;
pub const TCB_OBJ_BIT_SZ: usize = 10;

impl KernelObject for TcbObj {
    fn obj_type() -> ObjType { ObjType::Tcb }
}

impl Capability<TcbObj> {
    pub fn configure(&self, vspace_cap: Option<usize>, cspace_cap: Option<usize>) -> SysResult<()> {
        let info = MsgInfo::new(SyscallOp::TcbConfigure, 2);
        let mut args = [self.slot, vspace_cap.unwrap_or(0), cspace_cap.unwrap_or(0), 0, 0, 0];
        syscall(info, &mut args).map(|_|())
    }

    pub fn set_registers(&self, flags: usize, elr: usize, sp: usize) -> SysResult<()> {
        let info = MsgInfo::new(SyscallOp::TcbSetRegisters, 3);
        let mut args = [self.slot, flags, elr, sp, 0, 0];
        syscall(info, &mut args).map(|_|())
    }

    pub fn resume(&self) -> SysResult<()> {
        let info = MsgInfo::new(SyscallOp::TcbResume, 0);
        let mut args = [self.slot, 0, 0, 0, 0, 0];
        syscall(info, &mut args).map(|_|())
    }
}