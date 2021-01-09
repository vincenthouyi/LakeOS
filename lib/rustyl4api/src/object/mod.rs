use core::marker::PhantomData;

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

pub use cnode::{CNodeCap, CNodeObj, CNODE_DEPTH};
pub use endpoint::{EndpointObj, EpCap};
pub use interrupt::{InterruptCap, InterruptObj};
pub use monitor::MonitorObj;
pub use ram::{RamCap, RamObj};
pub use reply::{ReplyCap, ReplyObj};
pub use tcb::{TcbCap, TcbObj, TCB_OBJ_BIT_SZ, TCB_OBJ_SZ};
pub use untyped::UntypedObj;
pub use vtable::{VTableCap, VTableObj};

#[derive(Clone, Copy, PartialEq, Eq, Debug, FromPrimitive, ToPrimitive)]
pub enum ObjType {
    NullObj = 0,
    Untyped = 1,
    CNode = 2,
    Tcb = 3,
    Ram = 4,
    VTable = 5,
    Endpoint = 6,
    Reply = 7,
    Monitor = 8,
    Interrupt = 9,
}

impl Default for ObjType {
    fn default() -> Self {
        Self::NullObj
    }
}

#[derive(Debug, Clone)]
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
}

pub trait KernelObject {
    fn obj_type() -> ObjType;
}
