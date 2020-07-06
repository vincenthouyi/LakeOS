use super::*;
use sysapi::syscall::{SyscallOp, MsgInfo, RespInfo};

#[derive(Debug)]
pub enum MonitorObj {}

pub type MonitorCap<'a> = CapRef<'a, MonitorObj>;

impl<'a> MonitorCap<'a> {
    pub const fn mint() -> CapRaw {
        CapRaw::new(0, 0, 0, None, None, ObjType::Monitor)
    }

    pub fn debug_formatter(_f: &mut core::fmt::DebugStruct, _cap: &CapRaw) {
        return;
    }

    pub fn handle_invocation(&self, info: MsgInfo, tcb: &mut TcbObj) -> SysResult<()> {

        match info.get_label() {
            SyscallOp::MonitorMintUntyped => {
                if info.get_length() < 4 {
                    return Err(SysError::InvalidValue);
                }

                let slot = tcb.get_mr(1);
                let paddr = tcb.get_mr(2);
                let bit_size = tcb.get_mr(3);
                let is_device = tcb.get_mr(4) == 1;

                let cspace = tcb.cspace().unwrap();
                let cap = cspace.lookup_slot(slot).unwrap();

                if cap.get().cap_type() != ObjType::NullObj {
                    return Err(SysError::SlotIsNotEmpty);
                }

                cap.set(UntypedCap::mint(paddr, bit_size, is_device));

                tcb.set_respinfo(RespInfo::new(SysError::OK, 0));

                Ok(())
            }
            SyscallOp::CapIdentify => {
                tcb.set_mr(1, self.cap_type() as usize);

                tcb.set_respinfo(RespInfo::new(SysError::OK, 1));

                Ok(())
            }
            _ => { Err(SysError::UnsupportedSyscallOp) }
        }
    }
}
