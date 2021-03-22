use rustyl4api::error::SysResult;
use rustyl4api::syscall::{syscall, MsgInfo, SyscallOp};
use crate::objects::ObjType;

use super::{Capability, KernelObject};

#[derive(Debug)]
pub struct InterruptObj {}
pub type InterruptCap = Capability<InterruptObj>;

impl KernelObject for InterruptObj {
    fn obj_type() -> ObjType {
        ObjType::Interrupt
    }
}

impl Capability<InterruptObj> {
    pub fn attach_ep_to_irq(&self, ep_slot: usize, irq: usize) -> SysResult<()> {
        let info = MsgInfo::new(SyscallOp::InterruptAttachIrq, 2);

        let mut args = [self.slot(), ep_slot, irq, 0, 0, 0];
        syscall(info, &mut args).map(|_| ())
    }
}
