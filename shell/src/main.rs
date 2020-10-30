#![no_std]
#![no_main]

#![feature(once_cell)]

extern crate alloc;

#[macro_use] extern crate naive;

use rustyl4api::{kprintln};
use naive::io::{stdin, stdout};
use naive::rpc::RpcClient;
use naive::ep_server::EP_SERVER;
use conquer_once::spin::OnceCell;

mod shell;

pub fn ns_client() -> RpcClient {
    use rustyl4api::{process::ProcessCSpace, object::EpCap};
    static NS_CLIENT: OnceCell<RpcClient> = OnceCell::uninit();

    NS_CLIENT.try_get_or_init(|| {
        let ep_server = EP_SERVER.try_get().unwrap();
        let (ntf_badge, ntf_ep) = ep_server.derive_badged_cap().unwrap();
        RpcClient::connect(EpCap::new(ProcessCSpace::NameServer as usize), ntf_ep, ntf_badge).unwrap()
    }).unwrap().clone()
}

#[naive::main]
async fn main() -> () {
    use crate::alloc::string::ToString;
    kprintln!("shell process start");

    let mut client = ns_client();
    let stdio_cap = client.lookup_service("tty".to_string()).await;
    unsafe { 
        naive::io::stdio::STDOUT_CAP = stdio_cap.caps[0];
        naive::io::stdio::STDIN_CAP = stdio_cap.caps[0];
    }

    loop {
        shell::shell("test shell >").await;
        println!("Test shell exit, restarting...").await;
    }
}