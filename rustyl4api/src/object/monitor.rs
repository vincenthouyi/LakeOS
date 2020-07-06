use crate::object::{ObjType, UntypedObj};
use crate::error::SysResult;
use crate::syscall::{MsgInfo, SyscallOp, syscall};

use super::{Capability, KernelObject};

#[derive(Debug)]
pub struct MonitorObj {}

impl KernelObject for MonitorObj {
    fn obj_type() -> ObjType { ObjType::Monitor }
}

impl Capability<MonitorObj> {
    pub fn mint_untyped(&self, slot: usize, paddr: usize, bit_size: usize, is_device: bool) -> SysResult<Capability<UntypedObj>>
    {
        let info = MsgInfo::new(SyscallOp::MonitorMintUntyped, 4);
        let mut args = [self.slot, slot, paddr, bit_size, is_device as usize, 0];
        syscall(info, &mut args).map(|_| Capability::new(slot))
    }

}