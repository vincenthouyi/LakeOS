use core::fmt::{Debug, Error, Formatter};
use core::mem::size_of;

use super::*;
use crate::arch::trapframe::TrapFrame;
use crate::cspace::CSpace;
use crate::objects::NullCap;
use crate::syscall::{MsgInfo, RespInfo};
use crate::utils::tcb_queue::TcbQueueNode;
use crate::vspace::VSpace;

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
#[repr(align(512))]
#[derive(Default)]
pub struct TcbObj {
    pub tf: TrapFrame,
    cspace: CNodeEntry,
    vspace: CNodeEntry,
    reply_cap: CNodeEntry,
    time_slice: Cell<usize>,
    state: Cell<ThreadState>,
    sending_badge: Cell<usize>,
    pub node: TcbQueueNode,
}

impl Debug for TcbObj {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("TcbObj")
            .field("trapfram", &self.tf)
            .field("cspace", &self.cspace)
            .field("vspace", &self.vspace)
            .field("time_slice", &self.time_slice.get())
            .field("state", &self.state.get())
            .field("queue node", &self.node)
            .finish()
    }
}

pub const TCB_OBJ_SZ: usize = size_of::<TcbObj>().next_power_of_two();
pub const TCB_OBJ_BIT_SZ: usize = TCB_OBJ_SZ.trailing_zeros() as usize;
const_assert_eq!(TCB_OBJ_SZ, sysapi::object::TCB_OBJ_SZ);
const_assert_eq!(TCB_OBJ_BIT_SZ, sysapi::object::TCB_OBJ_BIT_SZ);

pub type TcbCap<'a> = CapRef<'a, TcbObj>;

impl TcbObj {
    pub const fn new() -> Self {
        Self {
            tf: TrapFrame::new(),
            cspace: Cell::new(NullCap::mint()),
            vspace: Cell::new(NullCap::mint()),
            reply_cap: Cell::new(NullCap::mint()),
            time_slice: Cell::new(0),
            state: Cell::new(ThreadState::Ready),
            sending_badge: Cell::new(0),
            node: TcbQueueNode::new(),
        }
    }

    pub fn configure_idle_thread(&mut self) {
        self.tf.configure_idle_thread()
    }

    pub fn install_cspace(&self, cspace: &CNodeCap) -> SysResult<()> {
        cspace.derive(&NullCap::try_from(&self.cspace)?)
    }

    pub fn install_vspace(&mut self, vspace: VTableCap) {
        let asid = (vspace.paddr() >> 12) & MASK!(16);
        vspace.set_mapped_vaddr_asid(0, asid, 1);
        let raw = vspace.raw();
        self.vspace.set(raw);
    }

    pub fn cspace(&self) -> SysResult<CSpace<'static>> {
        let cap = CNodeCap::try_from(&self.cspace).map_err(|_| SysError::CSpaceNotFound)?;
        Ok(CSpace(cap.as_object_mut()))
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
            let cpuid = crate::arch::cpuid() << 48;
            llvm_asm!("msr tpidrro_el0, $0"::"r"(cpuid | self.thread_id()));
            self.switch_vspace().unwrap_or(()); // explicitly ignore error for idle thread
            self.tf.restore();
        }
    }

    pub fn thread_id(&self) -> usize {
        ((self as *const _ as usize) & MASK!(48)) >> 10
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

    pub fn set_reply(&self, reply: Option<&TcbObj>) {
        match reply {
            None => self.reply_cap.set(NullCap::mint()),
            Some(tcb) => {
                let cap = ReplyCap::mint(tcb as *const _ as usize - crate::prelude::KERNEL_OFFSET);
                self.reply_cap.set(cap)
            }
        }
    }

    pub fn reply_cap(&self) -> Option<ReplyObj> {
        let cap = ReplyCap::try_from(&self.reply_cap).ok()?;
        Some(ReplyObj(cap.waiting_tcb()))
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

    pub fn set_timeslice(&self, ts: usize) {
        self.time_slice.set(ts);
    }

    pub fn timeslice(&self) -> usize {
        self.time_slice.get()
    }

    pub fn timeslice_sub(&self, t: usize) {
        let cur = self.timeslice();
        let ts = cur.saturating_sub(t);
        self.set_timeslice(ts);
    }

    pub fn sending_badge(&self) -> Option<usize> {
        let badge = self.sending_badge.get();
        if badge == 0 {
            None
        } else {
            Some(badge)
        }
    }

    pub fn set_sending_badge(&self, badge: usize) {
        self.sending_badge.set(badge)
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
        CapRaw::new(paddr, 0, 0, None, None, ObjType::Tcb)
    }

    pub fn identify(&self, tcb: &mut TcbObj) -> usize {
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
