#![no_std]
#![no_main]

#[macro_use] extern crate naive;
#[macro_use] extern crate rustyl4api;
#[macro_use] extern crate alloc;

mod console;
mod gpio;

use alloc::vec::Vec;
use alloc::boxed::Box;

use async_trait::async_trait;

use naive::ep_server::EP_SERVER;
use naive::rpc;
use naive::rpc::{
    RpcServer,
    WriteRequest, WriteResponse,
    ReadRequest, ReadResponse,
    RequestMemoryRequest, RequestMemoryResponse,
};
use naive::lmp::LmpListener;
use naive::io::AsyncWriteExt;
use naive::lmp::LmpListenerHandle;

use futures_util::StreamExt;
use conquer_once::spin::OnceCell;
use naive::{rpc::RpcClient};

pub fn ns_client() -> RpcClient {
    use rustyl4api::{process::ProcessCSpace, object::EpCap};
    static NS_CLIENT: OnceCell<RpcClient> = OnceCell::uninit();

    NS_CLIENT.try_get_or_init(|| {
        let ep_server = EP_SERVER.try_get().unwrap();
        let (ntf_badge, ntf_ep) = ep_server.derive_badged_cap().unwrap();
        RpcClient::connect(EpCap::new(ProcessCSpace::NameServer as usize), ntf_ep, ntf_badge).unwrap()
    }).unwrap().clone()
}

pub async fn request_memory(paddr: usize, size: usize, maybe_device: bool) -> usize {

    let mut client = ns_client();
    let ret = client.request_memory(paddr, size, maybe_device).await;
    *ret.caps.get(0).unwrap()
}

struct ConsoleApi;

#[async_trait]
impl naive::rpc::RpcRequestHandlers for ConsoleApi {
    async fn handle_write(&self, request: &WriteRequest) -> rpc::Result<(WriteResponse, Vec<usize>)> {
        let mut con = crate::console::console();
        let res = con.write(&request.buf).await;
        Ok((WriteResponse{result: res.unwrap() }, Vec::new()))
    }

    async fn handle_read(&self, request: &ReadRequest) -> rpc::Result<(ReadResponse, Vec<usize>)> {
        let read_len = request.len;
        let mut buf = Vec::new();
        let mut con_stream = crate::console::console();
        for _ in 0 .. read_len {
            if let Some(b) = con_stream.next().await {
                buf.push(b);
            } else {
                break;
            }
        }
        Ok((ReadResponse{buf: buf}, Vec::new()))
    }

    async fn handle_request_memory(&self, request: &RequestMemoryRequest) -> rpc::Result<(RequestMemoryResponse, Vec<usize>)> {
        use rustyl4api::object::RamObj;

        let cap = naive::space_manager::alloc_object_at::<RamObj>(request.paddr, request.size.trailing_zeros() as usize, request.maybe_device).unwrap().slot;
        let resp = RequestMemoryResponse{ result: 0};
        Ok((resp, [cap].to_vec()))
    }
}

#[naive::main]
async fn main() {
    use rustyl4api::object::interrupt::InterruptCap;
    use crate::alloc::string::ToString;
    gpio::init_gpio_server().await;
    console::console_server_init().await;

    let ep_server = EP_SERVER.try_get().unwrap();
    let con = console::console();
    let (irq_badge, irq_ep) = ep_server.derive_badged_cap().unwrap();
    let irq_rpc = ns_client().request_irq(pi::interrupt::Interrupt::Aux as usize).await;
    let irq_cap = InterruptCap::new(irq_rpc.caps[0]);
    irq_cap.attach_ep_to_irq(irq_ep.slot, pi::interrupt::Interrupt::Aux as usize).unwrap();
    ep_server.insert_notification(pi::interrupt::Interrupt::Aux as usize, Box::new(con.clone()));

    let (listen_badge, listen_ep) = ep_server.derive_badged_cap().unwrap();
    let listener = LmpListener::new(listen_ep.clone(), listen_badge);
    let listener = LmpListenerHandle::new(listener);
    ep_server.insert_event(listen_badge, Box::new(listener.clone()));

    let console_api = ConsoleApi{};
    let mut console_server = RpcServer::new(listener, console_api);

    let res = ns_client().register_service("tty".to_string(), listen_ep.slot).await;

    console_server.run().await;

    loop {}
}
