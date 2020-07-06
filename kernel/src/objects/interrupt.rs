use super::*;
use sysapi::syscall::{SyscallOp, MsgInfo, RespInfo};

#[derive(Debug)]
pub enum InterruptObj {}

pub type InterruptCap<'a> = CapRef<'a, InterruptObj>;

impl<'a> InterruptCap<'a> {
    pub const fn mint() -> CapRaw {
        CapRaw::new(0, 0, 0, None, None, ObjType::Interrupt)
    }

    pub fn debug_formatter(_f: &mut core::fmt::DebugStruct, _cap: &CapRaw) {
        return;
    }

    pub fn handle_invocation(&self, info: MsgInfo, tcb: &mut TcbObj) -> SysResult<()> {

        match info.get_label() {
            SyscallOp::InterruptAttachIrq => {
                let ep = tcb.get_mr(1);
                let irq = tcb.get_mr(2);

                let cspace = tcb.cspace().unwrap();
                let ep_slot = cspace.lookup_slot(ep)?;
                let ep_cap = EndpointCap::try_from(ep_slot)?;

                ep_cap.set_attach(super::Attach::Irq(irq));

                unsafe {
                    crate::interrupt::INTERRUPT_CONTROLLER.lock().attach_irq(irq, Cell::new(ep_cap.raw()));
                }

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
