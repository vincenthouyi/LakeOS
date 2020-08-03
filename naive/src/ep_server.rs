use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::collections::BTreeMap;

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

type EventHandlerCb = fn(&EpServer, IpcMessage, Option<usize>, Context) -> ();

#[derive(Clone, Copy)]
pub enum Context {
    Pointer(usize),
}

pub struct EpServer {
    event_handlers: OnceCell<Mutex<BTreeMap<usize, (EventHandlerCb, Context)>>>,
    ep: Ep
}

impl EpServer {
    pub const fn new(ep: EpCap) -> Self {
        Self {
            ep: Ep::from_unbadged(ep),
            event_handlers: OnceCell::uninit(),
        }
    }

    fn get_event_handlers(&self) -> MutexGuard<BTreeMap<usize, (EventHandlerCb, Context)>> {
        self.event_handlers
            .try_get_or_init(|| Mutex::new(BTreeMap::new())).unwrap()
            .lock()
    }

    pub fn derive_badged_cap(&self) -> Option<(usize, EpCap)> {
        self.ep.derive_badged_cap()
    }

    pub fn insert_event(&self, badge: usize, cb: EventHandlerCb, ctx: Context) {
        self.get_event_handlers()
            .insert(badge, (cb, ctx));
    }

    pub fn remove_event(&self, badge: usize) {
        self.get_event_handlers()
            .remove(&badge);
    }

    pub fn run(&self) {
        let mut recv_slot = gsm!().cspace_alloc().unwrap();
        let mut ret = self.ep.ep.receive(Some(recv_slot));
        while let Ok(IpcMessage::Message{payload, need_reply, cap_transfer, badge}) = ret {
            if let Some(b) = badge {
                let cb = self.get_event_handlers()
                             .get(&b)
                             .map(|cb| *cb);
                if let Some((cb, ctx)) = cb {
                    let cap_trans = if cap_transfer {
                        Some(recv_slot)
                    } else {
                        None
                    };
                    cb(self, ret.unwrap(), cap_trans, ctx);
                }
            } else {
                kprintln!("warning: receive unbadged message");
            }
            //TOO: leak previous alloced slot now. should find some other way...
            if cap_transfer {
                recv_slot = gsm!().cspace_alloc().unwrap();
            }
            ret = self.ep.ep.receive(Some(recv_slot));
        }
    }
}