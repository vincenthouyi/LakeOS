#![feature(decl_macro)]
#![feature(asm)]
#![feature(const_fn)]
#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate rustyl4api;

// mod rt;

use alloc::boxed::Box;
use alloc::vec::Vec;

use async_trait::async_trait;

use rustyl4api::object::CNodeCap;

use alloc::string::String;
use conquer_once::spin::OnceCell;
use hashbrown::HashMap;
use naive::lmp::LmpListenerHandle;
use naive::ns;
use naive::rpc::{
    self, LookupServiceRequest, LookupServiceResponse, RegisterServiceRequest,
    RegisterServiceResponse, RequestIrqRequest, RequestIrqResponse, RequestMemoryRequest,
    RequestMemoryResponse, RpcServer,
};
use spin::Mutex;

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
impl rpc::RpcRequestHandlers for InitThreadApi {
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
    ) -> rpc::Result<(RegisterServiceResponse, Vec<usize>)> {
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
    ) -> rpc::Result<(LookupServiceResponse, Vec<usize>)> {
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

struct InitFs;

impl InitFs {
    fn archive() -> cpio::NewcReader<'static> {
        unsafe {
            cpio::NewcReader::from_bytes(
                core::slice::from_raw_parts(0x40000000 as *const u8, 0x4000000)
            )
        }
    }

    pub fn get(&self, name: &str) -> Option<&'static [u8]> {
        Self::archive()
            .entries()
            .find(|e| e.name() == name)
            .map(|e| e.content())
    }
}

#[naive::main]
async fn main() {
    kprintln!("Init thread started");

    let ep_server = EP_SERVER.try_get().unwrap();
    let (listen_badge, listen_ep) = ep_server.derive_badged_cap().unwrap();

    let listener = LmpListenerHandle::new(listen_ep.clone(), listen_badge);
    ep_server.insert_event(listen_badge, listener.clone());

    let initfs = InitFs { };

    initfs.get("console")
        .map(|e| {
            naive::process::ProcessBuilder::new(e)
                .stdin(listen_ep.clone())
                .stdout(listen_ep.clone())
                .stderr(listen_ep.clone())
                .name_server(listen_ep.clone())
                .spawn()
                .expect("spawn process failed");
        })
        .expect("console binary not found");

    initfs.get("shell")
        .map(|e| {
            naive::process::ProcessBuilder::new(e)
                .stdin(listen_ep.clone())
                .stdout(listen_ep.clone())
                .stderr(listen_ep.clone())
                .name_server(listen_ep.clone())
                .spawn()
                .expect("spawn process failed");
        })
        .expect("shell binary not found");

    initfs.get("timer")
        .map(|e| {
            naive::process::ProcessBuilder::new(e)
                .stdin(listen_ep.clone())
                .stdout(listen_ep.clone())
                .stderr(listen_ep.clone())
                .name_server(listen_ep.clone())
                .spawn()
                .expect("spawn process failed");
        })
        .expect("timer binary not found");

    let rpc_api = InitThreadApi {};
    let mut rpc_server = RpcServer::new(listener, rpc_api);

    rpc_server.run().await;
}
