#![no_std]
#![no_main]

#[macro_use]
extern crate rustyl4api;
extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;

use async_trait::async_trait;

use naive::lmp::LmpListenerHandle;
use naive::ns::ns_client;
use naive::rpc::{self, ReadRequest, ReadResponse, RpcServer};
use naive::objects::{CapSlot, RamCap};

mod timer;

pub async fn request_memory(paddr: usize, size: usize, maybe_device: bool) -> Result<RamCap, ()> {
    let client = ns_client();
    let cap = client
        .lock()
        .request_memory(paddr, size, maybe_device)
        .await;
    cap
}

struct TimerApi;

#[async_trait]
impl naive::rpc::RpcRequestHandlers for TimerApi {
    async fn handle_read(&self, _request: &ReadRequest) -> rpc::Result<(ReadResponse, Vec<CapSlot>)> {
        let time = timer::current_time();
        let time_buf: [u8; 8] = unsafe { core::mem::transmute(time) };
        Ok((
            ReadResponse {
                buf: time_buf.to_vec(),
            },
            alloc::vec![],
        ))
    }
}

#[naive::main]
async fn main() {
    kprintln!("Timer started");

    timer::init_timer_server().await;

    let ep_server = EP_SERVER.try_get().unwrap();
    let (listen_badge, listen_ep) = ep_server.derive_badged_cap().unwrap();
    let listener = LmpListenerHandle::new(listen_ep.into(), listen_badge);
    let connector_ep = listener.derive_connector_ep().unwrap();
    ep_server.insert_event(listen_badge, listener.clone());

    let timer_api = TimerApi {};
    let timer_server = RpcServer::new(listener, timer_api);

    ns_client()
        .lock()
        .register_service("/dev/timer", connector_ep)
        .await
        .unwrap();

    timer_server.run().await;

    loop {}
}
