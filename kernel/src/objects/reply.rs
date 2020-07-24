use core::mem::size_of;
use core::fmt::{Debug, Formatter, Error};

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
        // let mut has_cap_transfer = false;
        let receiver = self.waiting_tcb();
        // let msglen = info.get_length();
        // for i in 1..=msglen {
        //     let data = sender.get_mr(i);
        //     receiver.set_mr(i, data);
        // }

        let recv_info = receiver.get_msginfo().unwrap();
        do_ipc(receiver, recv_info, sender, info, None, false, true)?;
        // // kprintln!("{} here recv_info {:?} send info {:?}", line!(), recv_info, info);
        // if recv_info.cap_transfer && info.cap_transfer {
        //     let recv_idx = receiver.get_mr(5);
        //     let recv_cspace = receiver.cspace()?;
        //     let recv_slot = recv_cspace.lookup_slot(recv_idx)?;
        //     let recv_cap = NullCap::try_from(recv_slot)?;

        //     let send_idx = sender.get_mr(5);
        //     let send_cspace = sender.cspace().unwrap();
        //     let send_slot = send_cspace.lookup_slot(send_idx)?;

        //     recv_cap.insert_raw(send_slot.get());
        //     send_slot.set(NullCap::mint());
        //     has_cap_transfer = true;
        // }

        receiver.set_state(ThreadState::Ready);
        receiver.set_sending_badge(0);
        // receiver.set_respinfo(RespInfo::ipc_resp(SysError::OK, msglen, has_cap_transfer, false, false));
        crate::SCHEDULER.push(receiver);
        if !will_recv {
            sender.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, 0));
            sender.set_reply(None);
        }

        Ok(())
    }

    pub fn identify(&self, tcb: &TcbObj) -> usize {
        tcb.set_mr(1, self.cap_type() as usize);
        1
    }

    pub fn debug_formatter(f: &mut core::fmt::DebugStruct, cap: &CapRaw) {
        // let c = Cell::new(*cap);
        // let c = TcbCap::try_from(&c).unwrap();
        // f.field("vaddr", &c.vaddr());
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