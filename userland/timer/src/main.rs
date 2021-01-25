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
use naive::rpc::{self, CurrentTimeRequest, CurrentTimeResponse, RpcServer};

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
    async fn handle_current_time(
        &self,
        _request: &CurrentTimeRequest,
    ) -> rpc::Result<(CurrentTimeResponse, Vec<usize>)> {
        let resp = CurrentTimeResponse {
            time: timer::current_time(),
        };
        Ok((resp, [].to_vec()))
    }
}

#[naive::main]
async fn main() {
    // use rustyl4api::object::interrupt::InterruptCap;
    use crate::alloc::string::ToString;

    kprintln!("Timer started");

    timer::init_timer_server().await;

    let ep_server = EP_SERVER.try_get().unwrap();
    // let (irq_badge, irq_ep) = ep_server.derive_badged_cap().unwrap();
    // let irq_cap_slot = ns_client().lock().request_irq(pi::interrupt::Interrupt::Timer1 as usize).await.unwrap();
    // let irq_cap = InterruptCap::new(irq_cap_slot);
    // irq_cap.attach_ep_to_irq(irq_ep.slot, pi::interrupt::Interrupt::Timer1 as usize).unwrap();
    // ep_server.insert_notification(pi::interrupt::Interrupt::Timer1 as usize, Box::new(TimerApi{}));

    let (listen_badge, listen_ep) = ep_server.derive_badged_cap().unwrap();
    let listener = LmpListenerHandle::new(listen_ep.clone(), listen_badge);
    ep_server.insert_event(listen_badge, listener.clone());

    let timer_api = TimerApi {};
    let mut timer_server = RpcServer::new(listener, timer_api);

    ns_client()
        .lock()
        .register_service("timer".to_string(), listen_ep.slot)
        .await
        .unwrap();

    timer_server.run().await;

    loop {}
}
