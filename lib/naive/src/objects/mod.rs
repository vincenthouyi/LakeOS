use core::marker::PhantomData;

use rustyl4api::syscall::{syscall, MsgInfo, SyscallOp};

mod cap_slot;
mod capref;
pub mod cnode;
pub mod endpoint;
pub mod identify;
pub mod interrupt;
pub mod monitor;
pub mod ram;
pub mod reply;
pub mod tcb;
pub mod untyped;
pub mod vtable;

pub use cap_slot::*;
pub use capref::*;
pub use cnode::{CNodeCap, CNodeObj, CNODE_DEPTH};
pub use endpoint::{EndpointObj, EpCap};
pub use interrupt::{InterruptCap, InterruptObj};
pub use monitor::{MonitorCap, MonitorObj};
pub use ram::{RamCap, RamObj};
pub use reply::{ReplyCap, ReplyObj};
pub use rustyl4api::objects::ObjType;
pub use tcb::{TcbCap, TcbObj, TCB_OBJ_BIT_SZ, TCB_OBJ_SZ};
pub use untyped::{UntypedCap, UntypedObj};
pub use vtable::{VTableCap, VTableObj};

#[derive(Debug)]
pub struct Capability<T: KernelObject> {
    pub slot: CapSlot,
    pub obj_type: PhantomData<T>,
}

impl<T: KernelObject> Capability<T> {
    pub const fn new(slot: CapSlot) -> Self {
        Self {
            slot: slot,
            obj_type: PhantomData,
        }
    }

    fn slot(&self) -> usize {
        self.slot.slot()
    }

    pub fn into_slot(self) -> CapSlot {
        /* Get inner slot without runinng destructor */
        let slot = self.slot();
        core::mem::forget(self);
        CapSlot::new(slot)
    }

    fn delete(&self) {
        let info = MsgInfo::new(SyscallOp::CNodeDelete, 0);
        let mut args = [self.slot(), 0, 0, 0, 0, 0];
        syscall(info, &mut args).map(|_| ()).unwrap();
    }
}

impl<T: KernelObject> core::ops::Drop for Capability<T> {
    fn drop(&mut self) {
        if self.slot() == 0 {
            return;
        }

        self.delete();
    }
}

pub trait KernelObject {
    fn obj_type() -> ObjType;
}
