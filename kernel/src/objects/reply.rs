use super::endpoint::do_ipc;
use super::*;
use crate::objects::{CapRef, TcbObj};
use crate::syscall::{MsgInfo, RespInfo};

// #[derive(Default)]
pub struct ReplyObj(pub *mut TcbObj);

impl ReplyObj {
    pub fn waiting_tcb(&self) -> &mut TcbObj {
        unsafe { &mut *self.0 }
    }

    pub fn handle_reply(
        &self,
        info: MsgInfo,
        sender: &mut TcbObj,
        will_recv: bool,
    ) -> SysResult<()> {
        let receiver = self.waiting_tcb();

        if let Some(_) = receiver.fault.take() {
            receiver.set_state(ThreadState::Ready);
            crate::SCHEDULER.get_mut().push(receiver);
            sender.set_reply(None);
            sender.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, 0));
        } else {
            let recv_info = receiver.get_msginfo().unwrap();
            do_ipc(receiver, recv_info, sender, info, None, false, true)?;

            receiver.set_state(ThreadState::Ready);
            receiver.set_sending_badge(0);
            crate::SCHEDULER.get_mut().push(receiver);
            sender.set_reply(None);
            if !will_recv {
                sender.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, 0));
            } else {
                receiver.set_reply(Some(sender));
                sender.set_state(ThreadState::Sending);
                sender.detach();
            }
        }

        Ok(())
    }
}

pub type ReplyCap<'a> = CapRef<'a, ReplyObj>;

impl<'a> ReplyCap<'a> {
    pub fn mint(paddr: usize) -> CapRaw {
        CapRaw::new(paddr, 0, 0, None, None, ObjType::Reply)
    }

    pub fn waiting_tcb(&self) -> &mut TcbObj {
        let paddr = self.raw.get().paddr;
        unsafe { &mut *((paddr + crate::prelude::KERNEL_OFFSET) as *mut TcbObj) }
    }

    pub fn identify(&self, tcb: &mut TcbObj) -> usize {
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

// impl<'a> core::ops::Deref for ReplyCap<'a> {
//     type Target = TcbObj;

//     fn deref(&self) -> &Self::Target {
//         unsafe{
//             &*((self.paddr() + KERNEL_OFFSET) as *const Self::Target)
//         }
//     }
// }
