use super::*;
use crate::vspace::VSpace;
use crate::arch::trapframe::TrapFrame;
use crate::syscall::{MsgInfo, RespInfo};
use crate::utils::tcb_queue::TcbQueueNode;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ThreadState {
    Ready,
    Sending,
    Receiving,
}

impl core::default::Default for ThreadState {
    fn default() -> Self {
        Self::Ready
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct TcbObj {
    pub tf: TrapFrame,
    pub cspace: CNodeEntry,
    pub vspace: CNodeEntry,
    pub time_slice: usize,
    pub state: ThreadState,
    pub node: TcbQueueNode,
}

pub const TCB_BIT_SIZE: u32 = core::mem::size_of::<TcbObj>()
                                .next_power_of_two()
                                .trailing_zeros();

pub type TcbCap<'a> = CapRef<'a, TcbObj>;

impl TcbObj {
    pub fn install_cspace(&self, cspace: &CNodeCap) -> SysResult<()> {
        cspace.derive(&NullCap::try_from(&self.cspace)?)
    }

    pub fn install_vspace(&self, vspace: VTableCap) {
        let asid = (vspace.paddr() >> 12) & MASK!(16);
        vspace.set_mapped_vaddr_asid(0, asid, 1);
        let raw = vspace.raw();
        self.vspace.set(raw);
    }

    pub fn cspace(&self) -> Option<CNodeCap> {
        CNodeCap::try_from(&self.cspace).ok()
    }

    pub fn vspace(&self) -> Option<VSpace> {
        let pgd = VTableCap::try_from(&self.vspace).ok()?;
        Some(VSpace::from_pgd(&pgd))
    }

    pub unsafe fn switch_vspace(&self) -> SysResult<()> {
        let pgd_cap = VTableCap::try_from(&self.vspace)?;
        let asid = self.asid()?;
        crate::arch::vspace::install_user_vspace(asid, pgd_cap.paddr());
        crate::arch::vspace::invalidateLocalTLB_ASID(asid);
        Ok(())
    }

    pub fn activate(&mut self) -> ! {
        unsafe {
            self.switch_vspace().unwrap();
            self.tf.restore();
        }
    }

    pub fn detach(&self) {
        self.node.detach()
    }

    pub fn get_mr(&self, idx: usize) -> usize {
        self.tf.get_mr(idx)
    }

    pub fn set_mr(&mut self, idx: usize, mr: usize) {
        self.tf.set_mr(idx, mr)
    }

    pub fn get_msginfo(&self) -> SysResult<MsgInfo> {
        self.tf.get_msginfo()
    }

    pub fn set_respinfo(&mut self, respinfo: RespInfo) {
        self.tf.set_respinfo(respinfo)
    }

    pub fn asid(&self) -> SysResult<usize> {
        // use PGD[28:12] bits as asid
        let pgd_cap = VTableCap::try_from(&self.vspace)?;
        Ok((pgd_cap.paddr() >> 12) & MASK!(16))
    }
}

/* Capability Entry Field Definition
 * -------------------------------------------------
 * |             paddr             |          |type|
 * |              52               |          | 4  |
 * -------------------------------------------------
 * |                                               |
 * |                                               |
 * -------------------------------------------------
 */
impl<'a> TcbCap<'a> {
    pub fn mint(paddr: usize) -> CapRaw {
        CapRaw::new(
            paddr,
            0,
            0,
            None,
            None,
            ObjType::Tcb
        )
    }

    pub fn handle_invocation(&mut self, info: MsgInfo, tcb: &mut TcbObj) -> SysResult<()> {
        use crate::syscall::SyscallOp;

        match info.get_label() {
            SyscallOp::TcbConfigure => {
                if info.get_length() < 2 {
                    return Err(SysError::InvalidValue);
                }
                let host_cspace = tcb.cspace().unwrap();

                let vspace_cap_idx = tcb.get_mr(1);
                let vspace_slot = host_cspace.lookup_slot(vspace_cap_idx)?;
                let vspace_cap = VTableCap::try_from(vspace_slot)?;
                let dst_vspace = NullCap::try_from(&self.vspace)?;
                vspace_cap.derive(&dst_vspace)?;

                let cspace_cap_idx = tcb.get_mr(2);
                let cspace_slot = host_cspace.lookup_slot(cspace_cap_idx)?;
                let cspace_cap = CNodeCap::try_from(cspace_slot)?;
                let dst_cspace = NullCap::try_from(&self.cspace)?;
                cspace_cap.derive(&dst_cspace)?;

                tcb.set_respinfo(RespInfo::new(SysError::OK, 0));
                Ok(())
            },
            SyscallOp::TcbSetRegisters => {
                if info.get_length() < 3 {
                    return Err(SysError::InvalidValue);
                }
                let reg_flags = tcb.get_mr(1);

                if reg_flags & 0b1000 == 0b1000 {
                    let elr = tcb.get_mr(2);
                    self.tf.set_elr(elr);
                }

                if reg_flags & 0b0100 == 0b0100 {
                    let sp = tcb.get_mr(3);
                    self.tf.set_sp(sp);
                }

                tcb.set_respinfo(RespInfo::new(SysError::OK, 0));
                Ok(())
            },
            SyscallOp::TcbResume => {
                crate::SCHEDULER.push(self);
                tcb.set_respinfo(RespInfo::new(SysError::OK, 0));
                Ok(())
            },
            SyscallOp::CapIdentify => {
                tcb.set_mr(1, self.cap_type() as usize);

                tcb.set_respinfo(RespInfo::new(SysError::OK, 1));

                Ok(())
            }
            _ => { Err(SysError::UnsupportedSyscallOp) }
        }
    }

    pub fn debug_formatter(f: &mut core::fmt::DebugStruct, cap: &CapRaw) {
        let c = Cell::new(*cap);
        let c = TcbCap::try_from(&c).unwrap();
        f.field("vaddr", &c.vaddr());
        return;
    }
}