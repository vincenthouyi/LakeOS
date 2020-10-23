use core::fmt;

use alloc::boxed::Box;

use conquer_once::spin::OnceCell;
use rustyl4api::object::EpCap;
use rustyl4api::process::ProcessCSpace;

use crate::urpc::{UrpcStream, UrpcStreamHandle};
use crate::ep_server::EP_SERVER;

fn stdio() -> UrpcStreamHandle {
    static STDIO: OnceCell<UrpcStreamHandle> = OnceCell::uninit(); 

    STDIO.try_get_or_init(|| {
        let ep_server = EP_SERVER.try_get().unwrap();
        let (ntf_badge, ntf_ep) = ep_server.derive_badged_cap().unwrap();
        let urpc = UrpcStream::connect(EpCap::new(ProcessCSpace::Stdio as usize), ntf_ep, ntf_badge).unwrap();
        let stdin = UrpcStreamHandle::from_stream(urpc);
        ep_server.insert_event(ntf_badge, Box::new(stdin.clone()));
        stdin
    }).unwrap().clone()
}

pub fn stdin() -> UrpcStreamHandle {
    stdio()
}

pub fn stdout() -> UrpcStreamHandle {
    stdio()
}

pub async fn _print(args: fmt::Arguments<'_>) {
    use futures_util::io::AsyncWriteExt;

    if let Err(e) = stdout().write_fmt(args).await {
        panic!("failed printing to stdout: {}", e);
    }
}