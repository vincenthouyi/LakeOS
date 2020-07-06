use core::mem::size_of;

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
    cspace: CNodeEntry,
    vspace: CNodeEntry,
    pub time_slice: usize,
    state: Cell<ThreadState>,
    pub node: TcbQueueNode,
}

pub const TCB_OBJ_SZ: usize = size_of::<TcbObj>().next_power_of_two();
pub const TCB_OBJ_BIT_SZ: usize = TCB_OBJ_SZ.trailing_zeros() as usize;
const_assert_eq!(TCB_OBJ_SZ, sysapi::object::TCB_OBJ_SZ);
const_assert_eq!(TCB_OBJ_BIT_SZ, sysapi::object::TCB_OBJ_BIT_SZ);

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

    pub fn cspace(&self) -> SysResult<CNodeCap> {
        CNodeCap::try_from(&self.cspace)
            .map_err(|_| SysError::CSpaceNotFound)
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

    pub fn set_mr(&self, idx: usize, mr: usize) {
        self.tf.set_mr(idx, mr)
    }

    pub fn get_msginfo(&self) -> SysResult<MsgInfo> {
        self.tf.get_msginfo()
    }

    pub fn set_respinfo(&self, respinfo: RespInfo) {
        self.tf.set_respinfo(respinfo)
    }

    pub fn asid(&self) -> SysResult<usize> {
        // use PGD[28:12] bits as asid
        let pgd_cap = VTableCap::try_from(&self.vspace)?;
        Ok((pgd_cap.paddr() >> 12) & MASK!(16))
    }

    pub fn configure(&self, cspace: Option<CNodeCap>, vspace: Option<VTableCap>) -> SysResult<()> {
        if let Some(vs) = vspace {
            let dst_vspace = NullCap::try_from(&self.vspace)?;
            vs.derive(&dst_vspace)?;
        }

        if let Some(cs) = cspace {
            let dst_cspace = NullCap::try_from(&self.cspace)?;
            cs.derive(&dst_cspace)?;
        }

        Ok(())
    }

    pub fn set_state(&self, state: ThreadState) {
        self.state.set(state)
    }

    pub fn state(&self) -> ThreadState {
        self.state.get()
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

    pub fn identify(&self, tcb: &TcbObj) -> usize {
        tcb.set_mr(1, self.cap_type() as usize);
        1
    }

    pub fn debug_formatter(f: &mut core::fmt::DebugStruct, cap: &CapRaw) {
        let c = Cell::new(*cap);
        let c = TcbCap::try_from(&c).unwrap();
        f.field("vaddr", &c.vaddr());
        return;
    }
}