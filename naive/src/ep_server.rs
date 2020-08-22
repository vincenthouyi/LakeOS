use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use alloc::sync::Arc;

use conquer_once::spin::OnceCell;
use spin::{Mutex, MutexGuard};

use rustyl4api::object::{EpCap};
use rustyl4api::ipc::{IpcMessage};
use crate::space_manager::gsm;

pub struct Ep {
    ep: EpCap,
    cur_badge: AtomicUsize,
}

impl Ep {
    pub const fn from_unbadged(ep: EpCap) -> Self {
        Self { ep, cur_badge: AtomicUsize::new(100) }
    }

    pub fn derive_badged_cap(&self) -> Option<(usize, EpCap)> {
        let slot = gsm!().cspace_alloc().unwrap();
        let badge = self.cur_badge.fetch_add(1, Ordering::Relaxed);
        self.ep.mint(slot, badge).unwrap();
        Some((badge, EpCap::new(slot)))
    }
}

pub struct EpServer {
    event_handlers: OnceCell<Mutex<BTreeMap<usize, Arc<Box<dyn EpMsgHandler>>>>>,
    ntf_handler: Mutex<[Option<Arc<Box<dyn EpNtfHandler>>>; 64]>,
    ep: Ep
}

// TODO: impl Sync and Send as a walkaround for now
unsafe impl Sync for EpServer { }
unsafe impl Send for EpServer { }

impl EpServer {
    pub const fn new(ep: EpCap) -> Self {
        Self {
            ep: Ep::from_unbadged(ep),
            event_handlers: OnceCell::uninit(),
            ntf_handler: Mutex::new([None; 64]),
        }
    }

    fn get_event_handlers(&self) -> MutexGuard<BTreeMap<usize, Arc<Box<dyn EpMsgHandler>>>> {
        self.event_handlers
            .try_get_or_init(|| Mutex::new(BTreeMap::new())).unwrap()
            .lock()
    }

    pub fn derive_badged_cap(&self) -> Option<(usize, EpCap)> {
        self.ep.derive_badged_cap()
    }

    pub fn insert_event(&self, badge: usize, cb: Box<dyn EpMsgHandler>) {
        self.get_event_handlers()
            .insert(badge, Arc::new(cb));
    }

    pub fn remove_event(&self, badge: usize) {
        self.get_event_handlers()
            .remove(&badge);
    }

    pub fn insert_notification(&self, ntf: usize, cb: Box<dyn EpNtfHandler>) {
        self.ntf_handler.lock()[ntf] = Some(Arc::new(cb));
    }

    pub fn run(&self) {
        let mut recv_slot = gsm!().cspace_alloc().unwrap();
        loop {
            let ret = self.ep.ep.receive(Some(recv_slot));
            match ret {
                Ok(IpcMessage::Message{payload, need_reply, cap_transfer, badge}) => {
                    if let Some(b) = badge {
                        let cb = self.get_event_handlers()
                                    .get(&b)
                                    .map(|cb| cb.clone());
                        if let Some(cb) = cb {
                            let cap_trans = if cap_transfer {
                                Some(recv_slot)
                            } else {
                                None
                            };
                            cb.handle_ipc(self, ret.unwrap(), cap_trans);
                        }
                    } else {
                        kprintln!("warning: receive unbadged message");
                    }
                    //TOO: leak previous alloced slot now. should find some other way...
                    if cap_transfer {
                        recv_slot = gsm!().cspace_alloc().unwrap();
                    }
                },
                Ok(IpcMessage::Notification(ntf_mask)) => {
                    let mut ntf_mask = ntf_mask;
                    while ntf_mask.trailing_zeros() != 64 {
                        let ntf = ntf_mask.trailing_zeros() as usize;
                        let cb = self.ntf_handler.lock()[ntf].clone();
                        if let Some(c) = cb {
                            c.handle_notification(self, ntf);
                        }
                        ntf_mask &= !(1 << ntf);
                    }
                }
                e => {
                    kprintln!("e {:?}", e);
                }
            }
        }
    }
}

pub trait EpMsgHandler {
    fn handle_ipc(&self, ep_server: &EpServer, msg: IpcMessage, cap_transfer_slot: Option<usize>) { }

    fn handle_fault(&self) { }
}

pub trait EpNtfHandler {
    fn handle_notification(&self, ep_server: &EpServer, ntf: usize) { }
}

pub static EP_SERVER: OnceCell<EpServer> = OnceCell::uninit();
pub fn ep_server() -> &'static EpServer {
    EP_SERVER.get().unwrap()
}