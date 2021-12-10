#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate lazy_static;

// mod rt;
mod devfs;
mod initfs;
mod rootfs;
mod vfs;

use core::ptr::NonNull;

use alloc::boxed::Box;
use alloc::vec::Vec;

use async_trait::async_trait;

use naive::lmp::LmpListener;
use naive::objects::{CapSlot, EpCap, IrqRef, KernelObject, MonitorRef, RamCap, RamObj};
use naive::rpc::{
    self, LookupServiceRequest, LookupServiceResponse, RegisterServiceRequest,
    RegisterServiceResponse, RequestIrqRequest, RequestIrqResponse, RequestMemoryRequest,
    RequestMemoryResponse, RpcServer, RpcServerHandler
};
use naive::ep_server::MsgReceiver;
use naive::space_manager::{copy_cap, gsm};
use rustyl4api::init::InitCSpaceSlot;
use spin::Mutex;

use log::trace;
use vfs::Vfs;

lazy_static! {
    pub static ref IRQ_CAP: IrqRef = IrqRef::from_slot_num(InitCSpaceSlot::IrqController as usize);
    pub static ref MONITOR_CAP: MonitorRef =
        MonitorRef::from_slot_num(InitCSpaceSlot::Monitor as usize);
}

pub fn allocate_frame_at(paddr: usize, _size: usize) -> Option<NonNull<u8>> {
    use rustyl4api::vspace::{Permission, FRAME_BIT_SIZE};

    let ram_obj = alloc_object_at::<RamObj>(paddr, FRAME_BIT_SIZE, true).unwrap();
    let vaddr = gsm!().insert_ram_at(ram_obj, 0, Permission::writable());

    NonNull::new(vaddr)
}

pub fn alloc_object_at<T: KernelObject>(
    paddr: usize,
    bit_sz: usize,
    maybe_device: bool,
) -> Option<RamCap> {
    // let monitor_cap = Capability::<MonitorObj>::new(CapSlot::new(Monitor as usize));
    let ut_slot = gsm!().cspace_alloc()?;
    let ut_cap = MONITOR_CAP
        .mint_untyped(ut_slot, paddr, bit_sz, maybe_device)
        .ok()?;
    let obj_slot = gsm!().cspace_alloc()?;
    let ret = ut_cap.retype_one(bit_sz, obj_slot).ok();
    core::mem::forget(ut_cap);
    ret
}

#[derive(Clone)]
struct InitThreadApi;

#[async_trait]
impl rpc::RpcRequestHandlers for InitThreadApi {
    async fn handle_request_memory(
        &self,
        request: &RequestMemoryRequest,
    ) -> naive::Result<(RequestMemoryResponse, Vec<CapSlot>)> {
        use naive::objects::RamObj;

        let cap = alloc_object_at::<RamObj>(
            request.paddr,
            request.size.trailing_zeros() as usize,
            request.maybe_device,
        )
        .unwrap();
        Ok((RequestMemoryResponse {}, alloc::vec![cap.into_slot()]))
    }

    async fn handle_request_irq(
        &self,
        _request: &RequestIrqRequest,
    ) -> naive::Result<(RequestIrqResponse, Vec<CapSlot>)> {
        let copy_cap = copy_cap(&IRQ_CAP).unwrap();
        Ok((RequestIrqResponse {}, alloc::vec![copy_cap.into_slot()]))
    }

    async fn handle_register_service(
        &self,
        request: &RegisterServiceRequest,
        mut cap: Vec<CapSlot>,
    ) -> naive::Result<(RegisterServiceResponse, Vec<CapSlot>)> {
        let slot = cap.pop().unwrap();
        VFS.lock()
            .publish(&request.name, EpCap::new(slot).into())
            .map_err(|_| naive::Error::InternalError)?;
        Ok((RegisterServiceResponse {}, alloc::vec![]))
    }

    async fn handle_lookup_service(
        &self,
        request: &LookupServiceRequest,
    ) -> naive::Result<(LookupServiceResponse, Vec<CapSlot>)> {
        let mut vfs_guard = VFS.lock();
        let node = vfs_guard
            .open(&request.name)
            .map_err(|_| naive::Error::InternalError)?;

        let ep = naive::space_manager::copy_cap(&node.cap).unwrap();
        Ok((LookupServiceResponse {}, alloc::vec![ep.into_slot()]))
    }
}

lazy_static! {
    static ref VFS: Mutex<Vfs> = Mutex::new(Vfs::new());
}

#[naive::main]
async fn main() {
    trace!("Init thread started");

    let receiver = MsgReceiver::new(&EP_SERVER);
    let listener = LmpListener::new(receiver);

    VFS.lock().mount("/", rootfs::RootFs::new()).unwrap();
    VFS.lock().mount("/dev", devfs::DevFs::new()).unwrap();
    VFS.lock().mount("/boot", initfs::InitFs::new()).unwrap();

    let initfs = initfs::InitFs::new();

    let console_proc = initfs
        .get(b"console")
        .map(|e| {
            naive::process::ProcessBuilder::new(e)
                .stdin(listener.derive_connector_ep().unwrap())
                .stdout(listener.derive_connector_ep().unwrap())
                .stderr(listener.derive_connector_ep().unwrap())
                .name_server(listener.derive_connector_ep().unwrap())
                .spawn()
                .expect("spawn process failed")
        })
        .expect("console binary not found");
    core::mem::forget(console_proc);

    let shell_proc = initfs
        .get(b"shell")
        .map(|e| {
            naive::process::ProcessBuilder::new(e)
                .stdin(listener.derive_connector_ep().unwrap())
                .stdout(listener.derive_connector_ep().unwrap())
                .stderr(listener.derive_connector_ep().unwrap())
                .name_server(listener.derive_connector_ep().unwrap())
                .spawn()
                .expect("spawn process failed")
        })
        .expect("shell binary not found");
    core::mem::forget(shell_proc);

    let timer_proc = initfs
        .get(b"timer")
        .map(|e| {
            naive::process::ProcessBuilder::new(e)
                .stdin(listener.derive_connector_ep().unwrap())
                .stdout(listener.derive_connector_ep().unwrap())
                .stderr(listener.derive_connector_ep().unwrap())
                .name_server(listener.derive_connector_ep().unwrap())
                .spawn()
                .expect("spawn process failed")
        })
        .expect("timer binary not found");
    core::mem::forget(timer_proc);

    let rpc_api = InitThreadApi {};
    let rpc_api = RpcServerHandler::new(rpc_api);
    let mut rpc_server = RpcServer::new(listener);

    rpc_server.run(rpc_api).await;
}
