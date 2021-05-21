#![no_std]
#![no_main]

extern crate naive;
#[macro_use]
extern crate rustyl4api;
extern crate alloc;

mod console;
mod gpio;

use alloc::boxed::Box;
use alloc::vec::Vec;

use async_trait::async_trait;

use naive::io::AsyncWriteExt;
use naive::lmp::LmpListener;
use naive::ns::ns_client;
use naive::rpc::{
    ReadRequest, ReadResponse, RpcServer,
    WriteRequest, WriteResponse,
};
use naive::objects::{CapSlot, RamCap};
use pi::interrupt::Interrupt;

use futures_util::StreamExt;

pub async fn request_memory(paddr: usize, size: usize, maybe_device: bool) -> Result<RamCap, ()> {
    let client = ns_client();
    let cap = client
        .await
        .lock()
        .request_memory(paddr, size, maybe_device)
        .await;
    Ok(cap.unwrap())
}

struct ConsoleApi;

#[async_trait]
impl naive::rpc::RpcRequestHandlers for ConsoleApi {
    async fn handle_write(
        &self,
        request: &WriteRequest,
    ) -> naive::Result<(WriteResponse, Vec<CapSlot>)> {
        let mut con = crate::console::console();
        let res = con.write(&request.buf).await;
        Ok((
            WriteResponse {
                result: res.unwrap(),
            },
            Vec::new(),
        ))
    }

    async fn handle_read(&self, request: &ReadRequest) -> naive::Result<(ReadResponse, Vec<CapSlot>)> {
        let read_len = request.len;
        let mut buf = Vec::new();
        let mut con_stream = crate::console::console();
        for _ in 0..read_len {
            if let Some(b) = con_stream.next().await {
                buf.push(b);
            } else {
                break;
            }
        }
        Ok((ReadResponse { buf: buf }, Vec::new()))
    }
}

#[naive::main]
async fn main() {
    kprintln!("console app start");
    gpio::init_gpio_server().await;
    console::console_server_init().await;

    let ep_server = &*EP_SERVER;
    let con = console::console();
    let (_irq_badge, irq_ep) = ep_server.derive_badged_cap().unwrap();
    let irq_cap = ns_client()
        .await
        .lock()
        .request_irq(Interrupt::Aux as usize)
        .await
        .unwrap();
    irq_cap
        .attach_ep_to_irq(irq_ep.slot.slot(), Interrupt::Aux as usize)
        .unwrap();
    ep_server.insert_notification(Interrupt::Aux as usize, con.clone());

    let receiver = EP_SERVER.derive_receiver();
    let listener = LmpListener::new(receiver);
    let connector_ep = listener.derive_connector_ep().unwrap();

    let console_api = ConsoleApi {};
    let console_server = RpcServer::new(listener, console_api);

    ns_client()
        .await
        .lock()
        .register_service("/dev/tty", connector_ep)
        .await
        .unwrap();

    console_server.run().await;

    loop {}
}
