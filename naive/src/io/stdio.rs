use core::fmt;

use alloc::boxed::Box;

use conquer_once::spin::OnceCell;
use rustyl4api::object::EpCap;
use rustyl4api::process::ProcessCSpace;

use crate::urpc::{UrpcStream, UrpcStreamHandle};
use crate::ep_server::EP_SERVER;

pub fn stdin() -> UrpcStreamHandle {
    static STDIN: OnceCell<UrpcStreamHandle> = OnceCell::uninit();

    STDIN.try_get_or_init(|| {
        let ep_server = EP_SERVER.try_get().unwrap();
        let (ntf_badge, ntf_ep) = ep_server.derive_badged_cap().unwrap();
        let urpc = UrpcStream::connect(EpCap::new(ProcessCSpace::Stdin as usize), ntf_ep, ntf_badge).unwrap();
        let stdin = UrpcStreamHandle::from_stream(urpc);
        ep_server.insert_event(ntf_badge, Box::new(stdin.clone()));
        stdin
    }).unwrap().clone()
}

pub fn stdout() -> UrpcStreamHandle {
    static STDOUT: OnceCell<UrpcStreamHandle> = OnceCell::uninit();

    STDOUT.try_get_or_init(|| {
        let ep_server = EP_SERVER.try_get().unwrap();
        let (ntf_badge, ntf_ep) = ep_server.derive_badged_cap().unwrap();
        let urpc = UrpcStream::connect(EpCap::new(ProcessCSpace::Stdout as usize), ntf_ep, ntf_badge).unwrap();
        let stdout = UrpcStreamHandle::from_stream(urpc);
        ep_server.insert_event(ntf_badge, Box::new(stdout.clone()));
        stdout
    }).unwrap().clone()
}

pub async fn _print(args: fmt::Arguments<'_>) {
    use futures_util::io::AsyncWriteExt;

    if let Err(e) = stdout().write_fmt(args).await {
        panic!("failed printing to stdout: {}", e);
    }
}