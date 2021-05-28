use alloc::sync::Arc;
use core::num::NonZeroUsize;
use core::sync::atomic::{AtomicUsize, Ordering};

use hashbrown::HashMap;
use spin::{Mutex, MutexGuard};

use crate::ep_receiver::EpReceiver;
use crate::ipc::{self, FaultMessage, IpcMessage};
use crate::objects::{EndpointObj, EpCap};
use crate::space_manager::{copy_cap_badged, gsm};

pub struct Ep {
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

    pub fn derive_badged_cap(&self) -> Option<(usize, EpCap)> {
        let badge = self.cur_badge.fetch_add(1, Ordering::Relaxed);
        let badged_ep = copy_cap_badged(&self.ep, NonZeroUsize::new(badge)).unwrap();
        Some((badge, badged_ep))
    }
}

pub struct EpServer {
    event_handlers: Mutex<HashMap<usize, Arc<dyn EpMsgHandler>>>,
    ntf_handler: Mutex<[Option<Arc<dyn EpNtfHandler>>; 64]>,
    fault_handlers: Mutex<HashMap<usize, Arc<dyn EpFaultHandler>>>,
    ep: Ep,
}

impl EpServer {
    pub fn new(ep: EpCap) -> Self {
        const INIT_NTF_HANDLER: Option<Arc<dyn EpNtfHandler>> = None;
        Self {
            ep: Ep::from_unbadged(ep),
            event_handlers: Mutex::new(HashMap::new()),
            ntf_handler: Mutex::new([INIT_NTF_HANDLER; 64]),
            fault_handlers: Mutex::new(HashMap::new()),
        }
    }

    fn get_event_handlers(&self) -> MutexGuard<HashMap<usize, Arc<dyn EpMsgHandler>>> {
        self.event_handlers.lock()
    }

    pub fn derive_badged_cap(&self) -> Option<(usize, EpCap)> {
        self.ep.derive_badged_cap()
    }

    pub fn derive_receiver(&self) -> EpReceiver {
        let (badge, ep) = self.derive_badged_cap().unwrap();
        let receiver = EpReceiver::new(ep.into(), badge);
        self.insert_event(badge, receiver.clone());

        receiver
    }

    pub fn insert_event<T: 'static + EpMsgHandler>(&self, badge: usize, cb: T) {
        self.get_event_handlers().insert(badge, Arc::new(cb));
    }

    pub fn remove_event(&self, badge: usize) {
        self.get_event_handlers().remove(&badge);
    }

    pub fn insert_notification<T: 'static + EpNtfHandler>(&self, ntf: usize, cb: T) {
        self.ntf_handler.lock()[ntf] = Some(Arc::new(cb));
    }

    pub fn insert_fault<T: 'static + EpFaultHandler>(&self, badge: usize, cb: T) {
        self.fault_handlers.lock().insert(badge, Arc::new(cb));
    }

    fn handle_ipc(&self, ipc_msg: IpcMessage) {
        match ipc_msg {
            IpcMessage::Message(msg) => {
                if let Some(b) = msg.badge {
                    let cb = self.get_event_handlers().get(&b).map(|cb| cb.clone());
                    if let Some(cb) = cb {
                        cb.handle_ipc(self, msg);
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
                    let cb = self.fault_handlers.lock().get(&b).map(|cb| cb.clone());
                    if let Some(cb) = cb {
                        cb.handle_fault(self, msg);
                    } else {
                        kprintln!("warning: receive message from unhandled badge {}", b);
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

pub trait EpMsgHandler: Send + Sync {
    fn handle_ipc(&self, _ep_server: &EpServer, _msg: ipc::Message) {}
}

pub trait EpNtfHandler: Send + Sync {
    fn handle_notification(&self, _ep_server: &EpServer, _ntf: usize) {}
}

lazy_static! {
    pub static ref EP_SERVER: EpServer = {
        let ep = gsm!().alloc_object::<EndpointObj>(12).unwrap();
        EpServer::new(ep)
    };
}

pub trait EpFaultHandler: Send + Sync {
    fn handle_fault(&self, _ep_server: &EpServer, _msg: FaultMessage) {}
}
