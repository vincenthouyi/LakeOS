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
use naive::rpc::{self, RpcServer, ReadRequest, ReadResponse};

mod timer;

pub async fn request_memory(paddr: usize, size: usize, maybe_device: bool) -> Result<usize, ()> {
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
    async fn handle_read(&self, _request: &ReadRequest) -> rpc::Result<(ReadResponse, Vec<usize>)> {
        let time = timer::current_time();
        let time_buf: [u8; 8] = unsafe {
            core::mem::transmute(time)
        };
        Ok((ReadResponse { buf: time_buf.to_vec() }, [].to_vec()))
    }

}

#[naive::main]
async fn main() {
    kprintln!("Timer started");

    timer::init_timer_server().await;

    let ep_server = EP_SERVER.try_get().unwrap();
    let (listen_badge, listen_ep) = ep_server.derive_badged_cap().unwrap();
    let listener = LmpListenerHandle::new(listen_ep.clone(), listen_badge);
    ep_server.insert_event(listen_badge, listener.clone());

    let timer_api = TimerApi {};
    let timer_server = RpcServer::new(listener, timer_api);

    ns_client()
        .lock()
        .register_service("/dev/timer", listen_ep.slot)
        .await
        .unwrap();

    timer_server.run().await;

    loop {}
}
