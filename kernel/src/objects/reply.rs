use super::*;
use crate::syscall::{MsgInfo, RespInfo};
use crate::objects::{CapRef, TcbObj};
use super::endpoint::do_ipc;

// #[derive(Default)]
pub struct ReplyObj([()]);

pub type ReplyCap<'a> = CapRef<'a, ReplyObj>;

impl<'a> ReplyCap<'a> {
    pub fn mint(paddr: usize) -> CapRaw {
        CapRaw::new(
            paddr,
            0,
            0,
            None,
            None,
            ObjType::Reply
        )
    }

    pub fn waiting_tcb(&self) -> &TcbObj {
        let paddr = self.raw.get().paddr;
        unsafe { &*((paddr + crate::prelude::KERNEL_OFFSET) as *const TcbObj) }
    }

    pub fn handle_reply(&self, info: MsgInfo, sender: &TcbObj, will_recv: bool) -> SysResult<()> {
        let receiver = self.waiting_tcb();

        let recv_info = receiver.get_msginfo().unwrap();
        do_ipc(receiver, recv_info, sender, info, None, false, true)?;

        receiver.set_state(ThreadState::Ready);
        receiver.set_sending_badge(0);
        crate::SCHEDULER.push(receiver);
        sender.set_reply(None);
        if !will_recv {
            sender.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, 0));
        } else {
            receiver.set_reply(Some(sender));
            sender.set_state(ThreadState::Sending);
            sender.detach();
        }

        Ok(())
    }

    pub fn identify(&self, tcb: &TcbObj) -> usize {
        tcb.set_mr(1, self.cap_type() as usize);
        1
    }

    pub fn debug_formatter(f: &mut core::fmt::DebugStruct, cap: &CapRaw) {
        let cap_raw = Cell::new(*cap);
        let cap = ReplyCap::try_from(&cap_raw).unwrap();
        f.field("waiting TCB", &cap.waiting_tcb());
        return;
    }
}

impl<'a> core::ops::Deref for ReplyCap<'a> {
    type Target = TcbObj;

    fn deref(&self) -> &Self::Target {
        unsafe{
            &*((self.paddr() + KERNEL_OFFSET) as *const Self::Target)
        }
    }
}