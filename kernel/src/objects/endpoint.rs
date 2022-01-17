use super::*;
use crate::objects::tcb::ThreadState;
use crate::syscall::{MsgInfo, RespInfo};
use crate::utils::tcb_queue::TcbQueue;
use core::num::NonZeroUsize;
use num_traits::FromPrimitive;
use sysapi::fault::Fault;
use sysapi::syscall::SyscallOp;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EpState {
    Free,
    Sending,
    Receiving,
    SignalPending,
}

impl core::default::Default for EpState {
    fn default() -> Self {
        Self::Free
    }
}

#[derive(Debug, Default)]
pub struct EndpointObj {
    queue: TcbQueue,
    signal: Cell<u64>,
    irq: Cell<u64>,
}

pub type EndpointCap<'a> = CapRef<'a, EndpointObj>;

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum AttachType {
    Unattached = 0,
    IrqController = 1,
}

#[derive(Clone, Copy, Debug)]
pub enum Attach {
    Unattached,
    Irq(usize),
}

/* Capability Entry Field Definition
 * -------------------------------------------------
 * |             paddr                    |    |W|R|
 * |              52                      |    |1|1|
 * -------------------------------------------------
 * |                    Badge                      |
 * |                      64                       |
 * -------------------------------------------------
 */

impl<'a> EndpointCap<'a> {
    pub const ADDR_MASK: usize = !MASK!(12); // TODO: check real obj type

    pub fn mint(paddr: usize, badge: usize) -> CapRaw {
        CapRaw::new(
            paddr,
            AttachType::Unattached as usize,
            badge,
            None,
            None,
            ObjType::Endpoint,
        )
    }

    pub fn set_attach(&self, attach: Attach) {
        let mut cap = self.raw();

        match attach {
            Attach::Unattached => {
                cap.arg1 = AttachType::Unattached as usize;
            }
            Attach::Irq(i) => {
                cap.arg1 = AttachType::IrqController as usize;
                cap.arg2 = i;

                let irq = self.irq.get() | (1 << i);
                self.irq.set(irq);
            }
        }

        self.raw.replace(cap);
    }

    pub fn get_attach(&self) -> Attach {
        let cap = self.raw();
        let attach_type = AttachType::from_usize(cap.arg1).unwrap();

        match attach_type {
            AttachType::Unattached => Attach::Unattached,
            AttachType::IrqController => Attach::Irq(cap.arg2),
        }
    }

    pub fn state(&self) -> EpState {
        if self.signal.get() != 0 {
            return EpState::SignalPending;
        }

        let head = self.queue.head();
        if head.is_none() {
            return EpState::Free;
        }
        let head = head.unwrap();

        match head.state() {
            ThreadState::Sending => EpState::Sending,
            ThreadState::Receiving => EpState::Receiving,
            s => {
                panic!("thread is not in state {:?}", s)
            }
        }
    }

    pub fn do_set_signal(&mut self, sig: u64) {
        let state = self.state();
        let signal = self.signal.get() | sig;
        self.signal.set(signal);

        if let EpState::Receiving = state {
            let receiver = self.queue.dequeue().unwrap();
            receiver.set_mr(1, self.signal.take() as usize);
            receiver.set_state(ThreadState::Ready);
            receiver.set_respinfo(RespInfo::new_notification());
            crate::SCHEDULER.get_mut().push(receiver);

            self.signal.set(0);
        }
    }

    pub fn handle_send(&self, info: MsgInfo, tcb: &mut TcbObj) -> SysResult<()> {
        match self.state() {
            EpState::Receiving => {
                let receiver = self.queue.dequeue().unwrap();
                let badge = self.badge();
                do_ipc(
                    receiver,
                    receiver.get_msginfo().unwrap(),
                    tcb,
                    info,
                    badge,
                    false,
                    false,
                )?;
                receiver.set_state(ThreadState::Ready);
                crate::SCHEDULER.get_mut().push(receiver);

                Ok(())
            }
            _ => {
                // TODO: check if send slot is legit if info.cap_transfer is set
                tcb.detach();
                tcb.set_state(ThreadState::Sending);
                let badge = self.badge();
                if let Some(b) = badge {
                    tcb.set_sending_badge(b);
                }
                self.queue.enqueue(tcb);
                Ok(())
            }
        }
    }

    pub fn handle_recv(&self, info: MsgInfo, tcb: &mut TcbObj) -> SysResult<()> {
        match self.state() {
            EpState::Free => {
                tcb.detach();
                tcb.set_state(ThreadState::Receiving);
                self.queue.enqueue(tcb);

                // TODO: check if recv slot is legit if info.cap_transfer is set
                if self.irq.get() != 0 {
                    unsafe {
                        crate::interrupt::INTERRUPT_CONTROLLER
                            .lock()
                            .listen_irq_mask(self.irq.get());
                    }
                }

                Ok(())
            }
            EpState::Receiving => {
                // TODO: check if recv slot is legit if info.cap_transfer is set
                tcb.detach();
                tcb.set_state(ThreadState::Receiving);
                self.queue.enqueue(tcb);
                Ok(())
            }
            EpState::SignalPending => {
                tcb.set_mr(1, self.signal.take() as usize);
                tcb.set_respinfo(RespInfo::new_notification());
                Ok(())
            }
            EpState::Sending => {
                let sender = self.queue.dequeue().unwrap();
                let badge = sender.sending_badge();
                if let Some(fault) = sender.fault.get() {
                    let buf = fault.as_ipc_message_buf();
                    tcb.set_mr(0, badge.unwrap_or(0));
                    for (i, reg) in buf.iter().enumerate() {
                        tcb.set_mr(i + 1, *reg);
                    }
                    tcb.set_respinfo(RespInfo::new_fault_resp(3, badge.is_some()));
                    tcb.set_reply(Some(sender));

                    Ok(())
                } else {
                    let sender_info = sender.get_msginfo().unwrap();
                    let is_call = sender_info.get_label() == SyscallOp::EndpointCall;
                    do_ipc(tcb, info, sender, sender_info, badge, is_call, false)?;
                    if is_call {
                        sender.detach();
                        tcb.set_reply(Some(sender));
                    } else {
                        sender.set_state(ThreadState::Ready);
                        crate::SCHEDULER.get_mut().push(sender);
                    }

                    Ok(())
                }
            }
        }
    }

    pub fn handle_call(&self, info: MsgInfo, sender: &mut TcbObj) -> SysResult<()> {
        match self.state() {
            EpState::Receiving => {
                let receiver = self.queue.dequeue().unwrap();
                let recv_info = receiver.get_msginfo().unwrap();
                let badge = self.badge();
                let ret = do_ipc(receiver, recv_info, sender, info, badge, true, false)?;

                sender.detach();
                receiver.set_state(ThreadState::Ready);
                receiver.set_reply(Some(sender));
                crate::SCHEDULER.get_mut().push(receiver);

                Ok(ret)
            }
            _ => {
                sender.detach();
                sender.set_state(ThreadState::Sending);
                let badge = self.badge();
                if let Some(b) = badge {
                    sender.set_sending_badge(b);
                }
                self.queue.enqueue(sender);
                Ok(())
            }
        }
    }

    pub fn send_fault_ipc(&self, sender: &mut TcbObj, fault: Fault) -> SysResult<()> {
        sender.fault.set(Some(fault));
        sender.set_state(ThreadState::Sending);
        sender.detach();

        match self.state() {
            EpState::Receiving => {
                let receiver = self.queue.dequeue().unwrap();

                let buf = fault.as_ipc_message_buf();
                let badge = self.badge();
                receiver.set_mr(0, badge.unwrap_or(0));
                for (i, reg) in buf.iter().enumerate() {
                    receiver.set_mr(i + 1, *reg);
                }

                receiver.set_respinfo(RespInfo::new_fault_resp(3, badge.is_some()));
                receiver.set_state(ThreadState::Ready);
                receiver.set_reply(Some(sender));
                crate::SCHEDULER.get_mut().push(receiver);

                Ok(())
            }
            _ => {
                let badge = self.badge();
                if let Some(b) = badge {
                    sender.set_sending_badge(b);
                }
                self.queue.enqueue(sender);
                Ok(())
            }
        }
    }

    pub fn derive(&self, dst: &NullCap) -> SysResult<()> {
        dst.raw.set(self.raw());
        Ok(())
    }

    pub fn badge(&self) -> Option<usize> {
        let b = self.raw().arg2;
        if b == 0 {
            None
        } else {
            Some(b)
        }
    }

    pub fn set_badge(&self, badge: usize) {
        self.raw().arg2 = badge;
    }

    pub fn derive_badged(&self, badge: Option<NonZeroUsize>) -> CapRaw {
        if badge.is_some() {
            EndpointCap::mint(self.paddr().0, badge.map(|b| b.get()).unwrap_or(0))
        } else {
            self.raw()
        }
    }

    pub fn identify(&self, tcb: &mut TcbObj) -> usize {
        tcb.set_mr(1, self.cap_type() as usize);
        1
    }

    pub fn debug_formatter(_f: &mut core::fmt::DebugStruct, _cap: &CapRaw) {
        return;
    }
}

pub fn do_ipc(
    recv: &mut TcbObj,
    recv_info: MsgInfo,
    send: &mut TcbObj,
    send_info: MsgInfo,
    badge: Option<usize>,
    is_call: bool,
    is_reply: bool,
) -> SysResult<()> {
    let mut has_cap_trans = false;

    let msglen = send_info.get_length();
    for i in 1..=msglen {
        let data = send.get_mr(i);
        recv.set_mr(i, data);
    }
    if let Some(b) = badge {
        recv.set_mr(0, b);
    }
    if recv_info.cap_transfer && send_info.cap_transfer {
        let recv_cspace = recv.cspace()?;
        let recv_idx = recv.get_mr(5);
        let recv_slot = recv_cspace.lookup_slot(recv_idx)?;

        let send_cspace = send.cspace()?;
        let send_idx = send.get_mr(5);
        let send_slot = send_cspace.lookup_slot(send_idx)?;

        let recv_cap = NullCap::try_from(recv_slot)?;
        recv_cap.insert_raw(send_slot.get());
        send_slot.set(NullCap::mint());
        has_cap_trans = true;
    }

    recv.set_respinfo(RespInfo::ipc_resp(
        SysError::OK,
        msglen,
        has_cap_trans,
        is_call,
        badge.is_some(),
    ));
    if !is_call && !is_reply {
        send.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, 0));
    }

    Ok(())
}
