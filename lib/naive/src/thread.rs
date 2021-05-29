use crate::ep_server::{EP_SERVER, FaultReceiver};
use crate::objects::{TcbCap, TcbObj};
use crate::space_manager::{gsm, ROOT_CNODE_CAP, ROOT_VNODE_CAP};

pub struct Thread {
    _tcb: TcbCap,
    _fault_receiver: FaultReceiver,
}

pub fn spawn(entry: fn() -> !) -> Thread {
    use rustyl4api::vspace::{Permission, FRAME_SIZE};

    let npages = 4;
    let tcb = gsm!().alloc_object::<TcbObj>(12).unwrap();

    let stack_base = gsm!()
        .map_frame_at(0, 0, FRAME_SIZE * npages, Permission::writable())
        .unwrap() as usize;
    let fault_receiver = EP_SERVER.derive_fault_receiver().unwrap();
    tcb.configure(
        Some(&ROOT_VNODE_CAP),
        Some(&ROOT_CNODE_CAP),
        Some(&fault_receiver.badged_ep()),
    )
    .expect("Error Configuring TCB");

    tcb.set_registers(0b1100, entry as usize, stack_base + FRAME_SIZE * npages)
        .expect("Error Setting Registers");
    tcb.resume().expect("Error Resuming TCB");
    Thread { _tcb: tcb, _fault_receiver: fault_receiver }
}
