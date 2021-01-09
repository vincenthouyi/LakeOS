#![feature(decl_macro)]
#![feature(asm)]
#![feature(const_fn)]
#![no_std]
#![no_main]

extern crate alloc;
extern crate futures_util;
extern crate naive;
#[macro_use]
extern crate rustyl4api;

// mod console;
// mod gpio;
// mod timer;
// mod rt;

use alloc::boxed::Box;
use alloc::vec::Vec;

use async_trait::async_trait;

use rustyl4api::object::CNodeCap;

use alloc::string::String;
use conquer_once::spin::OnceCell;
use hashbrown::HashMap;
use naive::lmp::{LmpListener, LmpListenerHandle};
use naive::ns;
use naive::rpc;
use naive::rpc::*;
use spin::Mutex;

const SHELL_ELF: &'static [u8] = include_bytes!("../build/shell");
const CONSOLE_ELF: &'static [u8] = include_bytes!("../build/console");
const TIMER_ELF: &'static [u8] = include_bytes!("../build/timer");

// fn timer_test() {
//     for i in 0..5 {
//         println!("timer {}: {}", i, timer::current_time());
//         timer::spin_sleep_ms(1000);
//     }

//     // works now, but we don't have interrupt handling at the moment
// //    system_timer::tick_in(1000);
// }

pub struct NameServer {
    pub services: Mutex<HashMap<String, usize>>,
}

pub fn name_server() -> &'static NameServer {
    static NAME_SERVER: OnceCell<NameServer> = OnceCell::uninit();

    NAME_SERVER
        .try_get_or_init(|| NameServer {
            services: Mutex::new(HashMap::new()),
        })
        .unwrap()
}

struct InitThreadApi;

#[async_trait]
impl naive::rpc::RpcRequestHandlers for InitThreadApi {
    async fn handle_request_memory(
        &self,
        request: &RequestMemoryRequest,
    ) -> rpc::Result<(RequestMemoryResponse, Vec<usize>)> {
        use rustyl4api::object::RamObj;

        let cap = naive::space_manager::alloc_object_at::<RamObj>(
            request.paddr,
            request.size.trailing_zeros() as usize,
            request.maybe_device,
        )
        .unwrap()
        .slot;
        let resp = RequestMemoryResponse { result: 0 };
        Ok((resp, [cap].to_vec()))
    }

    async fn handle_request_irq(
        &self,
        _request: &RequestIrqRequest,
    ) -> rpc::Result<(RequestIrqResponse, Vec<usize>)> {
        let copy_slot = naive::space_manager::gsm!().cspace_alloc().unwrap();
        let cspace = CNodeCap::new(rustyl4api::init::InitCSpaceSlot::InitCSpace as usize);
        cspace
            .cap_copy(
                copy_slot,
                rustyl4api::init::InitCSpaceSlot::IrqController as usize,
            )
            .unwrap();
        let resp = RequestIrqResponse { result: 0 };
        Ok((resp, [copy_slot].to_vec()))
    }

    async fn handle_register_service(
        &self,
        request: &RegisterServiceRequest,
        cap: Vec<usize>,
    ) -> Result<(RegisterServiceResponse, Vec<usize>)> {
        name_server()
            .services
            .lock()
            .insert(request.name.clone(), cap[0]);
        let resp = RegisterServiceResponse {
            result: ns::Error::Success,
        };
        Ok((resp, [].to_vec()))
    }

    async fn handle_lookup_service(
        &self,
        request: &LookupServiceRequest,
    ) -> Result<(LookupServiceResponse, Vec<usize>)> {
        let services = name_server().services.lock();
        let cap = services.get(&request.name);
        if let Some(c) = cap {
            let resp = LookupServiceResponse {
                result: ns::Error::Success,
            };
            Ok((resp, [*c].to_vec()))
        } else {
            let resp = LookupServiceResponse {
                result: ns::Error::ServiceNotFound,
            };
            Ok((resp, [].to_vec()))
        }
    }
}

#[naive::main]
async fn main() {
    kprintln!("Init thread started");

    let ep_server = EP_SERVER.try_get().unwrap();
    let (listen_badge, listen_ep) = ep_server.derive_badged_cap().unwrap();

    let listener = LmpListener::new(listen_ep.clone(), listen_badge);
    let listener = LmpListenerHandle::new(listener);
    ep_server.insert_event(listen_badge, Box::new(listener.clone()));

    naive::process::ProcessBuilder::new(&CONSOLE_ELF)
        .stdin(listen_ep.clone())
        .stdout(listen_ep.clone())
        .stderr(listen_ep.clone())
        .name_server(listen_ep.clone())
        .spawn()
        .expect("spawn process failed");

    naive::process::ProcessBuilder::new(&SHELL_ELF)
        .stdin(listen_ep.clone())
        .stdout(listen_ep.clone())
        .stderr(listen_ep.clone())
        .name_server(listen_ep.clone())
        .spawn()
        .expect("spawn process failed");

    naive::process::ProcessBuilder::new(&TIMER_ELF)
        .stdin(listen_ep.clone())
        .stdout(listen_ep.clone())
        .stderr(listen_ep.clone())
        .name_server(listen_ep.clone())
        .spawn()
        .expect("spawn process failed");

    let rpc_api = InitThreadApi {};
    let mut rpc_server = RpcServer::new(listener, rpc_api);

    rpc_server.run().await;
}
