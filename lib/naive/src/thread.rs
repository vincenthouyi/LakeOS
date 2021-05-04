use rustyl4api::fault::Fault;

use crate::objects::{TcbObj, TcbCap, EpCap, ReplyCap, CapSlot};
use crate::space_manager::{gsm, ROOT_CNODE_CAP, ROOT_VNODE_CAP};
use crate::ep_server::{EP_SERVER, EpFaultHandler, EpServer};
use crate::ipc::FaultMessage;

pub struct Thread {
    _tcb : TcbCap,
}

pub fn spawn(entry: fn() -> !) -> Thread {
    use rustyl4api::vspace::{Permission, FRAME_SIZE};

    let npages = 4;
    let tcb = gsm!().alloc_object::<TcbObj>(12).unwrap();

    let stack_base = gsm!()
        .map_frame_at(0, 0, FRAME_SIZE * npages, Permission::writable())
        .unwrap() as usize;
    let (fault_ep_badge, fault_ep) = EP_SERVER.derive_badged_cap().unwrap();
    tcb.configure(Some(&ROOT_VNODE_CAP), Some(&ROOT_CNODE_CAP), Some(&fault_ep))
        .expect("Error Configuring TCB");

    let fault_handler = ThreadFaultHandler {
        _fault_ep: fault_ep
    };
    EP_SERVER.insert_fault(fault_ep_badge, fault_handler);

    tcb.set_registers(0b1100, entry as usize, stack_base + FRAME_SIZE * npages)
        .expect("Error Setting Registers");
    tcb.resume().expect("Error Resuming TCB");
    Thread {
        _tcb: tcb
    }
}

struct ThreadFaultHandler {
    _fault_ep: EpCap,
}

impl EpFaultHandler for ThreadFaultHandler {
    fn handle_fault(&self, _ep_server: &EpServer, msg: FaultMessage) {
        let faultinfo = match msg.info {
            Fault::DataFault(f) => {
                f
            }
            Fault::PrefetchFault(f) => {
                f
            }
        };
        let fault_addr = faultinfo.address;
        kprintln!("recv fault message {:?}, fault address {:x}", msg, fault_addr);

        let reply = ReplyCap::new(CapSlot::new(0));
        reply.reply(&[], None).unwrap();
    }
}