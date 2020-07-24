use core::sync::atomic::{AtomicUsize, AtomicPtr, AtomicU8, Ordering};

use rustyl4api::object::{EpCap, RamCap, ReplyCap};

use crate::space_manager::gsm;
use crate::io::Result;

use super::{UrpcStream, Role};


pub struct UrpcListener {
    listen_badge: usize,
    listen_ep: EpCap,
}

static CUR_BADGE: AtomicUsize = AtomicUsize::new(100);

impl UrpcListener {
    pub fn bind(listen_ep: EpCap, listen_badge: usize) -> Result<Self> {
        Ok(Self { listen_badge, listen_ep })
    }

    pub fn accept_with(&self, c_ntf_ep: EpCap) -> Result<(UrpcStream, usize)> {
        use rustyl4api::vspace::Permission;

        /* Mint a new badged EP and send back to client */
        let badge_ep_slot = gsm!().cspace_alloc().unwrap();
        let conn_badge = CUR_BADGE.fetch_add(1, Ordering::Relaxed);
        self.listen_ep.mint(badge_ep_slot, conn_badge).unwrap();
        let ret = self.listen_ep.reply_receive(&[], Some(badge_ep_slot)).unwrap();

        let buf_cap = RamCap::new(badge_ep_slot);
        let buf_ptr = gsm!().insert_ram_at(buf_cap.clone(), 0, Permission::writable());

        let stream = UrpcStream::new(
            Role::Server, c_ntf_ep, buf_cap, buf_ptr
        );

        Ok((stream, conn_badge))
    }

    pub fn accept(&self) -> Result<(UrpcStream, usize)> {

        /* Waiting for incoming connection request and store incoming EP */
        let c_ntf_ep_slot = gsm!().cspace_alloc().unwrap();
        let ret = self.listen_ep.receive(Some(c_ntf_ep_slot)).unwrap();
        let c_ntf_ep = EpCap::new(c_ntf_ep_slot);

        self.accept_with(c_ntf_ep)
    }
}
