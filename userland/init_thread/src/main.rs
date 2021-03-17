#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate rustyl4api;

// mod rt;
mod devfs;
mod initfs;
mod vfs;

use alloc::boxed::Box;
use alloc::vec::Vec;

use async_trait::async_trait;

use naive::objects::{EpCap, InterruptCap};

use conquer_once::spin::OnceCell;
use naive::lmp::LmpListenerHandle;
use naive::ns;
use naive::rpc::{
    self, LookupServiceRequest, LookupServiceResponse, RegisterServiceRequest,
    RegisterServiceResponse, RequestIrqRequest, RequestIrqResponse, RequestMemoryRequest,
    RequestMemoryResponse, RpcServer,
};
use naive::space_manager::copy_cap;
use spin::Mutex;

struct InitThreadApi;

#[async_trait]
impl rpc::RpcRequestHandlers for InitThreadApi {
    async fn handle_request_memory(
        &self,
        request: &RequestMemoryRequest,
    ) -> rpc::Result<(RequestMemoryResponse, Vec<usize>)> {
        use naive::objects::RamObj;

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
        let irq_cap = InterruptCap::new(rustyl4api::init::InitCSpaceSlot::IrqController as usize);
        let copy_cap = copy_cap(&irq_cap).unwrap();
        let resp = RequestIrqResponse { result: 0 };
        Ok((resp, [copy_cap.slot].to_vec()))
    }

    async fn handle_register_service(
        &self,
        request: &RegisterServiceRequest,
        cap: Vec<usize>,
    ) -> rpc::Result<(RegisterServiceResponse, Vec<usize>)> {
        let ret = vfs().lock().publish(&request.name, EpCap::new(cap[0]));
        let resp = RegisterServiceResponse {
            result: {
                if ret.is_ok() {
                    ns::Error::Success
                } else {
                    ns::Error::ServiceNotFound
                }
            },
        };
        Ok((resp, [].to_vec()))
    }

    async fn handle_lookup_service(
        &self,
        request: &LookupServiceRequest,
    ) -> rpc::Result<(LookupServiceResponse, Vec<usize>)> {
        let mut vfs_guard = vfs().lock();
        let res = vfs_guard.open(&request.name);
        if let Ok(node) = res {
            let resp = LookupServiceResponse {
                result: ns::Error::Success,
            };
            Ok((resp, [node.cap].to_vec()))
        } else {
            let resp = LookupServiceResponse {
                result: ns::Error::ServiceNotFound,
            };
            Ok((resp, [].to_vec()))
        }
    }
}

use vfs::Vfs;
pub fn vfs() -> &'static Mutex<Vfs> {
    static VFS: OnceCell<Mutex<Vfs>> = OnceCell::uninit();

    VFS.try_get_or_init(|| Mutex::new(Vfs::new())).unwrap()
}

#[naive::main]
async fn main() {
    kprintln!("Init thread started");

    let ep_server = EP_SERVER.try_get().unwrap();
    let (listen_badge, listen_ep) = ep_server.derive_badged_cap().unwrap();

    let listener = LmpListenerHandle::new(listen_ep, listen_badge);
    ep_server.insert_event(listen_badge, listener.clone());

    vfs().lock().mount("/", initfs::InitFs::new()).unwrap();
    vfs().lock().mount("/dev", devfs::DevFs::new()).unwrap();

    let initfs = initfs::InitFs::new();

    initfs
        .get(b"console")
        .map(|e| {
            naive::process::ProcessBuilder::new(e)
                .stdin(listener.derive_connector_ep().unwrap())
                .stdout(listener.derive_connector_ep().unwrap())
                .stderr(listener.derive_connector_ep().unwrap())
                .name_server(listener.derive_connector_ep().unwrap())
                .spawn()
                .expect("spawn process failed");
        })
        .expect("console binary not found");

    initfs
        .get(b"shell")
        .map(|e| {
            naive::process::ProcessBuilder::new(e)
                .stdin(listener.derive_connector_ep().unwrap())
                .stdout(listener.derive_connector_ep().unwrap())
                .stderr(listener.derive_connector_ep().unwrap())
                .name_server(listener.derive_connector_ep().unwrap())
                .spawn()
                .expect("spawn process failed");
        })
        .expect("shell binary not found");

    initfs
        .get(b"timer")
        .map(|e| {
            naive::process::ProcessBuilder::new(e)
                .stdin(listener.derive_connector_ep().unwrap())
                .stdout(listener.derive_connector_ep().unwrap())
                .stderr(listener.derive_connector_ep().unwrap())
                .name_server(listener.derive_connector_ep().unwrap())
                .spawn()
                .expect("spawn process failed");
        })
        .expect("timer binary not found");

    let rpc_api = InitThreadApi {};
    let rpc_server = RpcServer::new(listener, rpc_api);

    rpc_server.run().await;
}
