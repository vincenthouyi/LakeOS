#![no_std]
#![no_main]

#![feature(once_cell)]

extern crate alloc;

#[macro_use] extern crate naive;

use alloc::sync::Arc;

use rustyl4api::{kprintln};
use naive::io::{stdin, stdout};
use naive::rpc::RpcClient;
use naive::ep_server::EP_SERVER;
use conquer_once::spin::OnceCell;
use spin::Mutex;

mod shell;

pub fn ns_client() -> Arc<Mutex<RpcClient>> {
    use rustyl4api::{process::ProcessCSpace, object::EpCap};
    static NS_CLIENT: OnceCell<Arc<Mutex<RpcClient>>> = OnceCell::uninit();

    NS_CLIENT.try_get_or_init(|| {
        let ep_server = EP_SERVER.try_get().unwrap();
        let (ntf_badge, ntf_ep) = ep_server.derive_badged_cap().unwrap();
        let inner = RpcClient::connect(EpCap::new(ProcessCSpace::NameServer as usize), ntf_ep, ntf_badge).unwrap();
        Arc::new(Mutex::new(inner))
    }).unwrap().clone()
}

#[naive::main]
async fn main() -> () {
    use crate::alloc::string::ToString;
    kprintln!("shell process start");

    let mut stdio_cap_slot = None;
    
    while let None = stdio_cap_slot {
        stdio_cap_slot = ns_client()
            .lock()
            .lookup_service("tty".to_string())
            .await
            .ok();
    }
    unsafe {
        naive::io::stdio::STDOUT_CAP = stdio_cap_slot.unwrap();
        naive::io::stdio::STDIN_CAP = stdio_cap_slot.unwrap();
    }

    loop {
        shell::shell("test shell >").await;
        println!("Test shell exit, restarting...").await;
    }
}