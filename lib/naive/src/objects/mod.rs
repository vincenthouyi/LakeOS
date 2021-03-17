use core::marker::PhantomData;

use rustyl4api::error::SysResult;
use rustyl4api::syscall::{MsgInfo, SyscallOp, syscall};

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

pub use rustyl4api::objects::{ObjType};
pub use cnode::{CNodeCap, CNodeObj, CNODE_DEPTH};
pub use endpoint::{EndpointObj, EpCap};
pub use interrupt::{InterruptCap, InterruptObj};
pub use monitor::MonitorObj;
pub use ram::{RamCap, RamObj};
pub use reply::{ReplyCap, ReplyObj};
pub use tcb::{TcbCap, TcbObj, TCB_OBJ_BIT_SZ, TCB_OBJ_SZ};
pub use untyped::UntypedObj;
pub use vtable::{VTableCap, VTableObj};

#[derive(Debug)]
pub struct Capability<T: KernelObject> {
    pub slot: usize,
    pub obj_type: PhantomData<T>,
}

impl<T: KernelObject> Capability<T> {
    pub const fn new(slot: usize) -> Self {
        Self {
            slot: slot,
            obj_type: PhantomData,
        }
    }

    pub fn derive(&self, dst_cptr: usize) -> SysResult<()> {
        let info = MsgInfo::new(SyscallOp::Derive, 1);
        let mut args = [self.slot, dst_cptr, 0, 0, 0, 0];
        syscall(info, &mut args).map(|_| ())
    }
}

pub trait KernelObject {
    fn obj_type() -> ObjType;
}
