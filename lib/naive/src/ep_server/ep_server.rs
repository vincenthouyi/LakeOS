use alloc::boxed::Box;
use core::num::NonZeroUsize;
use core::sync::atomic::{AtomicUsize, Ordering};

use hashbrown::HashMap;
use spin::RwLock;

use crate::ipc::{FaultMessage, IpcMessage, Message};
use crate::objects::{EpCap};
use crate::space_manager::{copy_cap_badged, gsm};

pub struct BadgedEp {
    ep: EpCap,
    badge: usize,
}

impl BadgedEp {
    pub fn new(ep: EpCap, badge: usize) -> Self {
        Self { ep, badge }
    }

    pub fn badge(&self) -> usize {
        self.badge
    }

    pub fn ep(&self) -> &EpCap {
        &self.ep
    }
}

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

    pub fn derive_badged_cap(&self) -> Option<BadgedEp> {
        let badge = self.alloc_badge()?;
        let badged_ep = copy_cap_badged(&self.ep, NonZeroUsize::new(badge))?;
        Some(BadgedEp::new(badged_ep, badge))
    }
}

pub trait MessageHandler: Send + Sync {
    fn handle_message(&self, ep_server: &EpServer, badge: usize, message: Message);
}

pub trait NotificationHandler: Send + Sync {
    fn handle_notification(&self, ep_server: &EpServer, notification: usize);
}

pub trait FaultHandler: Send + Sync {
    fn handle_fault(&self, ep_server: &EpServer, badge: usize, fault: FaultMessage);
}

pub struct EpServer {
    msg_handlers: RwLock<HashMap<usize, Box<dyn MessageHandler>>>,
    ntf_handler: RwLock<[Option<Box<dyn NotificationHandler>>; 64]>,
    fault_handlers: RwLock<HashMap<usize, Box<dyn FaultHandler>>>,
    ep: Ep,
}

impl EpServer {
    pub fn new(ep: EpCap) -> Self {
        const INIT_NTF_HANDLER: Option<Box<dyn NotificationHandler>> = None;
        Self {
            ep: Ep::from_unbadged(ep),
            msg_handlers: RwLock::new(HashMap::new()),
            ntf_handler: RwLock::new([INIT_NTF_HANDLER; 64]),
            fault_handlers: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_badged_ep(&self, badge: usize) -> EpCap {
        self.ep.get_badged_ep(badge)
    }

    pub fn handle_message<T: 'static + MessageHandler>(&self, msg_handler: T) -> Option<BadgedEp> {
        let badged = self.ep.derive_badged_cap()?;
        self.msg_handlers
            .write()
            .insert(badged.badge(), Box::new(msg_handler));
        Some(badged)
    }

    pub fn handle_notification<T: 'static + NotificationHandler>(&self, ntf: usize, cb: T) -> Option<BadgedEp> {
        let badged = self.ep.derive_badged_cap()?;
        self.ntf_handler.write()[ntf] = Some(Box::new(cb));
        Some(badged)
    }

    pub fn handle_fault<T: 'static + FaultHandler>(&self, fault_handler: T) -> Option<BadgedEp> {
        let badged = self.ep.derive_badged_cap()?;
        self.fault_handlers
            .write()
            .insert(badged.badge(), Box::new(fault_handler));
        Some(badged)
    }

    pub fn remove_message_handler(&self, badge: usize) {
        self.msg_handlers.write().remove(&badge);
    }

    pub fn remove_notification_handler(&self, badge: usize) {
        self.ntf_handler.write()[badge] = None;
    }

    pub fn remove_fault_handler(&self, badge: usize) {
        self.fault_handlers.write().remove(&badge);
    }

    fn handle_ipc(&self, ipc_msg: IpcMessage) {
        match ipc_msg {
            IpcMessage::Message(msg) => {
                if let Some(b) = msg.badge {
                    if let Some(handler) = self.msg_handlers.read().get(&b) {
                        handler.handle_message(self, b, msg);
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
                    let cb = &self.ntf_handler.read()[ntf];
                    if let Some(c) = cb {
                        c.handle_notification(self, ntf);
                    }
                    ntf_mask &= !(1 << ntf);
                }
            }
            IpcMessage::Fault(msg) => {
                if let Some(b) = msg.badge {
                    if let Some(handler) = self.fault_handlers.read().get(&b) {
                        // let reply = ReplyCap::new(CapSlot::new(0));
                        handler.handle_fault(self, b, msg);
                    } else {
                        log::warn!("receive fault from unhandled badge {}", b);
                    }
                } else {
                    log::warn!("receive unbadged fault message");
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
            } else if let Err(e) = ret {
                log::error!("Receiving error {:?}", e);
                break;
            }
        }
    }
}
