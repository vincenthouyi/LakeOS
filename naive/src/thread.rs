use rustyl4api::object::{TcbObj, RamObj};
use rustyl4api::init::InitCSpaceSlot::{InitCSpace, InitL1PageTable};
use rustyl4api::process::ProcessCSpace::{RootCNodeCap, RootVNodeCap};

use crate::space_manager::gsm;

pub fn spawn(entry: fn() -> !) {
    use rustyl4api::vspace::{FRAME_SIZE, Permission};

    let tcb = gsm!().alloc_object::<TcbObj>(12)
                        .unwrap();

    let stack_ram = gsm!().alloc_object::<RamObj>(12)
                              .unwrap();

    let stack_base = gsm!().insert_ram_at(stack_ram, 0, Permission::writable()) as usize;
    tcb.configure(Some(RootVNodeCap as usize), Some(RootCNodeCap as usize))
       .expect("Error Configuring TCB");
    tcb.set_registers(0b1100,entry as usize, stack_base + FRAME_SIZE)
       .expect("Error Setting Registers");
    tcb.resume()
       .expect("Error Resuming TCB");
}