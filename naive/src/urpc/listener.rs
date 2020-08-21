use rustyl4api::object::{EpCap, RamCap};

use crate::space_manager::gsm;
use crate::io::Result;

use super::{UrpcStream, Role};


pub struct UrpcListener {
    listen_badge: usize,
    listen_ep: EpCap,
}

impl UrpcListener {
    pub fn bind(listen_ep: EpCap, listen_badge: usize) -> Result<Self> {
        Ok(Self { listen_badge, listen_ep })
    }

    pub fn accept_with(&self, c_ntf_ep: EpCap, s_ntf_ep: EpCap) -> Result<UrpcStream> {
        use rustyl4api::vspace::Permission;

        let ret = self.listen_ep.reply_receive(&[], Some(s_ntf_ep.slot)).unwrap();

        let buf_cap = RamCap::new(s_ntf_ep.slot);
        let buf_ptr = gsm!().insert_ram_at(buf_cap.clone(), 0, Permission::writable());

        let stream = UrpcStream::new(
            Role::Server, c_ntf_ep, buf_cap, buf_ptr
        );

        Ok(stream)
    }
}
