use alloc::sync::Arc;
use core::num::NonZeroUsize;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::{Waker};

use hashbrown::HashMap;
use spin::{Mutex, MutexGuard};
use crossbeam_queue::{ArrayQueue, SegQueue};

use crate::ipc::{FaultMessage, IpcMessage, Message};
use crate::objects::{EpCap, ReplyCap, CapSlot};
use crate::space_manager::{copy_cap_badged, gsm};

use super::{MsgReceiver, FaultReceiver};

struct Ep {
    ep: EpCap,
    cur_badge: AtomicUsize,
}

impl Ep {
    pub const fn from_unbadged(ep: EpCap) -> Self {
        Self {
            ep,
            cur_badge: AtomicUsize::new(100),
        }
    }

    pub fn get_badged_ep(&self, badge: usize) -> EpCap {
        copy_cap_badged(&self.ep, NonZeroUsize::new(badge)).unwrap()
    }

    pub fn alloc_badge(&self) -> Option<usize> {
        Some(self.cur_badge.fetch_add(1, Ordering::Relaxed))
    }

    pub fn derive_badged_cap(&self) -> Option<(usize, EpCap)> {
        let badge = self.alloc_badge()?;
        let badged_ep = copy_cap_badged(&self.ep, NonZeroUsize::new(badge))?;
        Some((badge, badged_ep))
    }
}

pub struct MsgHandler {
    pub waker: Arc<SegQueue<Waker>>,
    pub buf: Arc<ArrayQueue<Message>>,
}

impl MsgHandler {
    pub fn handle_msg(&self, msg: Message) {
        self.buf.push(msg).unwrap();
        while let Ok(waker) = self.waker.pop() {
            waker.wake();
        }
    }
}

pub struct FaultHandler {
    pub waker: Arc<SegQueue<Waker>>,
    pub buf: Arc<ArrayQueue<(FaultMessage, ReplyCap)>>,
}

impl FaultHandler {
    pub fn handle_fault(&self, fault: FaultMessage, reply: ReplyCap) {
        self.buf.push((fault, reply)).unwrap();
        while let Ok(waker) = self.waker.pop() {
            waker.wake();
        }
    }
}

pub struct EpServer {
    msg_handlers: Mutex<HashMap<usize, MsgHandler>>,
    ntf_handler: Mutex<[Option<Arc<dyn EpNtfHandler>>; 64]>,
    fault_handlers: Mutex<HashMap<usize, FaultHandler>>,
    ep: Ep,
}

impl EpServer {
    pub fn new(ep: EpCap) -> Self {
        const INIT_NTF_HANDLER: Option<Arc<dyn EpNtfHandler>> = None;
        Self {
            ep: Ep::from_unbadged(ep),
            msg_handlers: Mutex::new(HashMap::new()),
            ntf_handler: Mutex::new([INIT_NTF_HANDLER; 64]),
            fault_handlers: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_msg_handlers(&self) -> MutexGuard<HashMap<usize, MsgHandler>> {
        self.msg_handlers.lock()
    }

    pub fn get_fault_handlers(&self) -> MutexGuard<HashMap<usize, FaultHandler>> {
        self.fault_handlers.lock()
    }

    pub fn get_badged_ep(&self, badge: usize) -> EpCap {
        self.ep.get_badged_ep(badge)
    }

    pub fn derive_badged_cap(&self) -> Option<(usize, EpCap)> {
        self.ep.derive_badged_cap()
    }

    pub fn derive_receiver(&'static self) -> Option<MsgReceiver> {
        let badge = self.ep.alloc_badge()?;
        let handler = MsgHandler {
            waker: Arc::new(SegQueue::new()),
            buf: Arc::new(ArrayQueue::new(10)),
        };
        self.msg_handlers.lock().insert(badge, handler);
        let receiver = MsgReceiver::new(badge, self);

        Some(receiver)
    }

    pub fn derive_fault_receiver(&'static self) -> Option<FaultReceiver> {
        let badge = self.ep.alloc_badge()?;
        let handler = FaultHandler {
            waker: Arc::new(SegQueue::new()),
            buf: Arc::new(ArrayQueue::new(10)),
        };
        self.get_fault_handlers().insert(badge, handler);
        let receiver = FaultReceiver::new(badge, self);

        Some(receiver)
    }

    pub fn remove_message_handler(&self, badge: usize) {
        self.get_msg_handlers().remove(&badge);
    }

    pub fn remove_fault_handler(&self, badge: usize) {
        self.get_fault_handlers().remove(&badge);
    }

    pub fn insert_notification<T: 'static + EpNtfHandler>(&self, ntf: usize, cb: T) {
        self.ntf_handler.lock()[ntf] = Some(Arc::new(cb));
    }

    fn handle_ipc(&self, ipc_msg: IpcMessage) {
        match ipc_msg {
            IpcMessage::Message(msg) => {
                if let Some(b) = msg.badge {
                    if let Some(handler) = self.get_msg_handlers().get(&b) {
                        handler.handle_msg(msg);
                    } else {
                        kprintln!("warning: receive message from unhandled badge {}", b);
                    }
                } else {
                    kprintln!("warning: receive unbadged message");
                }
            }
            IpcMessage::Notification(ntf_mask) => {
                let mut ntf_mask = ntf_mask;
                while ntf_mask.trailing_zeros() != 64 {
                    let ntf = ntf_mask.trailing_zeros() as usize;
                    let cb = &self.ntf_handler.lock()[ntf];
                    if let Some(c) = cb {
                        c.handle_notification(self, ntf);
                    }
                    ntf_mask &= !(1 << ntf);
                }
            }
            IpcMessage::Fault(msg) => {
                if let Some(b) = msg.badge {
                    if let Some(handler) = self.get_fault_handlers().get(&b) {
                        let reply = ReplyCap::new(CapSlot::new(0));
                        handler.handle_fault(msg, reply);
                    } else {
                        kprintln!("warning: receive fault from unhandled badge {}", b);
                    }
                } else {
                    kprintln!("warning: receive unbadged fault message");
                }
            }
            IpcMessage::Invalid => {
                kprintln!("Receiving invalid message");
            }
        }
    }

    pub fn run(&self) {
        loop {
            let recv_slot = gsm!().cspace_alloc().unwrap();
            let ret = self.ep.ep.receive(Some(recv_slot));
            if let Ok(r) = ret {
                self.handle_ipc(r);
            }
        }
    }
}

pub trait EpNtfHandler: Send + Sync {
    fn handle_notification(&self, _ep_server: &EpServer, _ntf: usize) {}
}
