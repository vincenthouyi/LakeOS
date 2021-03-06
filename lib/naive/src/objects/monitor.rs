use crate::objects::{CapSlot, ObjType, TcbCap, UntypedObj};
use rustyl4api::error::SysResult;
use rustyl4api::syscall::{syscall, MsgInfo, SyscallOp};

use super::{Capability, KernelObject};

#[derive(Debug)]
pub struct MonitorObj {}
pub type MonitorCap = Capability<MonitorObj>;

impl KernelObject for MonitorObj {
    fn obj_type() -> ObjType {
        ObjType::Monitor
    }
}

impl Capability<MonitorObj> {
    pub fn mint_untyped(
        &self,
        slot: CapSlot,
        paddr: usize,
        bit_size: usize,
        is_device: bool,
    ) -> SysResult<Capability<UntypedObj>> {
        let info = MsgInfo::new(SyscallOp::MonitorMintUntyped, 4);
        let mut args = [
            self.slot(),
            slot.slot(),
            paddr,
            bit_size,
            is_device as usize,
            0,
        ];
        //TODO:slot
        syscall(info, &mut args).map(|_| Capability::new(slot))
    }

    pub fn insert_tcb_to_cpu(&self, tcb: &TcbCap, cpu: usize) -> SysResult<()> {
        let info = MsgInfo::new(SyscallOp::MonitorInsertTcbToCpu, 2);
        let mut args = [self.slot(), tcb.slot(), cpu, 0, 0, 0];
        syscall(info, &mut args).map(|_| ())
    }
}
