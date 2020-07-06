use super::*;
use num_traits::FromPrimitive;
use crate::utils::tcb_queue::TcbQueue;
use crate::syscall::{MsgInfo, RespInfo};
use crate::objects::tcb::ThreadState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EpState {
    Free,
    Sending,
    Receiving,
    SignalPending,
}

impl core::default::Default for EpState {
    fn default() -> Self {
        Self:: Free
    }
}

#[derive(Debug, Default)]
pub struct EndpointObj {
    queue: TcbQueue,
    signal: Cell<u64>,
}

pub type EndpointCap<'a> = CapRef::<'a, EndpointObj>;

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum AttachType {
    Unattached    = 0,
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

    pub fn mint(paddr: usize) -> CapRaw {
        CapRaw::new(
            paddr,
            AttachType::Unattached as usize,
            0,
            None,
            None,
            ObjType::Endpoint
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
            ThreadState::Receiving  => EpState::Receiving,
            s => { panic!("thread is not in state {:?}", s) }
        }
    }

    pub fn do_set_signal(&mut self, sig: u64) {
        let state = self.state();
        let signal = self.signal.get() | sig;
        self.signal.set(signal);

        if let EpState::Receiving = state {
            let receiver = self.queue.dequeue().unwrap();
            receiver.set_mr(1, self.signal.get() as usize);
            receiver.set_state(ThreadState::Ready);
            receiver.set_respinfo(RespInfo::new_notification());
            crate::SCHEDULER.push(receiver);

            self.signal.set(0);
        }
    }

    pub fn handle_send(&self, info: MsgInfo, tcb: &TcbObj) -> SysResult<()> {
        match self.state() {
            EpState::Receiving => {
                let receiver = self.queue.dequeue().unwrap();
                let msglen = info.get_length();
                for i in 1..msglen {
                    let data = tcb.get_mr(i);
                    receiver.set_mr(i, data);
                }
                receiver.set_state(ThreadState::Ready);
                receiver.set_respinfo(RespInfo::new(SysError::OK, msglen));
                crate::SCHEDULER.push(receiver);

                tcb.set_respinfo(RespInfo::new(SysError::OK, 0));

                Ok(())
            }
            _ => {
                tcb.detach();
                tcb.set_state(ThreadState::Sending);
                self.queue.enqueue(tcb);
                Ok(())
            }
        }
    }

    pub fn handle_recv(&self, _: MsgInfo, tcb: &TcbObj) -> SysResult<()> {
        match self.state() {
            EpState::Free => {

                tcb.detach();
                tcb.set_state(ThreadState::Receiving);
                self.queue.enqueue(tcb);

                if let Attach::Irq(irq) = self.get_attach() {
                    unsafe {
                        crate::interrupt::INTERRUPT_CONTROLLER.lock().listen_irq(irq);
                    }
                }

                Ok(())
            }
            EpState::Receiving => {
                tcb.detach();
                tcb.set_state(ThreadState::Receiving);
                self.queue.enqueue(tcb);
                Ok(())
            }
            EpState::SignalPending => {
                tcb.set_mr(1, self.signal.get() as usize);
                tcb.set_respinfo(RespInfo::new_notification());
                Ok(())
            }
            EpState::Sending => {
                let sender = self.queue.dequeue().unwrap();
                let info = sender.get_msginfo().unwrap();
                let msglen = info.get_length();
                for i in 1..=msglen {
                    let data = sender.get_mr(i);
                    tcb.set_mr(i, data);
                }
                sender.set_state(ThreadState::Ready);
                sender.set_respinfo(RespInfo::new(SysError::OK, 0));
                crate::SCHEDULER.push(sender);

                tcb.set_respinfo(RespInfo::new(SysError::OK, msglen));

                Ok(())
            }
        }
    }

    pub fn identify(&self, tcb: &TcbObj) -> usize {
        tcb.set_mr(1, self.cap_type() as usize);
        1
    }

    pub fn debug_formatter(_f: &mut core::fmt::DebugStruct, _cap: &CapRaw) {
        return;
    }
}
