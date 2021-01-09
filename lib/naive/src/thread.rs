use rustyl4api::object::TcbObj;
use rustyl4api::process::ProcessCSpace::{RootCNodeCap, RootVNodeCap};

use crate::space_manager::gsm;

pub fn spawn(entry: fn() -> !) {
    use rustyl4api::vspace::{Permission, FRAME_SIZE};

    let npages = 4;
    let tcb = gsm!().alloc_object::<TcbObj>(12).unwrap();

    let stack_base = gsm!()
        .map_frame_at(0, 0, FRAME_SIZE * npages, Permission::writable())
        .unwrap() as usize;
    tcb.configure(Some(RootVNodeCap as usize), Some(RootCNodeCap as usize))
        .expect("Error Configuring TCB");
    tcb.set_registers(0b1100, entry as usize, stack_base + FRAME_SIZE * npages)
        .expect("Error Setting Registers");
    tcb.resume().expect("Error Resuming TCB");
}
