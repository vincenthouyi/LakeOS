#![no_std]
#![no_main]

#[macro_use]
extern crate rustyl4api;
extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;

use async_trait::async_trait;

use naive::lmp::LmpListener;
use naive::ns::ns_client;
use naive::rpc::{ReadRequest, ReadResponse, RpcServer};
use naive::objects::{CapSlot, RamCap};

mod timer;

pub async fn request_memory(paddr: usize, size: usize, maybe_device: bool) -> Result<RamCap, ()> {
    let client = ns_client();
    let cap = client
        .await
        .lock()
        .request_memory(paddr, size, maybe_device)
        .await;
    Ok(cap.unwrap())
}

struct TimerApi;

#[async_trait]
impl naive::rpc::RpcRequestHandlers for TimerApi {
    async fn handle_read(&self, _request: &ReadRequest) -> naive::Result<(ReadResponse, Vec<CapSlot>)> {
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

    let receiver = EP_SERVER.derive_receiver();
    let listener = LmpListener::new(receiver);
    let connector_ep = listener.derive_connector_ep().unwrap();

    let timer_api = TimerApi {};
    let timer_server = RpcServer::new(listener, timer_api);

    ns_client()
        .await
        .lock()
        .register_service("/dev/timer", connector_ep)
        .await
        .unwrap();

    timer_server.run().await;

    loop {}
}
