use core::marker::PhantomData;

mod cnode;
mod tcb;
mod ram;
mod vtable;
mod endpoint;
mod untyped;
mod monitor;
mod interrupt;
pub mod identify;

pub use cnode::{CNodeObj, CNodeCap, CNODE_DEPTH};
pub use tcb::{TcbObj, TCB_OBJ_SZ, TCB_OBJ_BIT_SZ};
pub use ram::{RamObj, RamCap};
pub use endpoint::EndpointObj;
pub use untyped::UntypedObj;
pub use monitor::MonitorObj;
pub use interrupt::InterruptObj;
pub use vtable::{VTableObj, VTableCap};

#[derive(Clone, Copy, PartialEq, Eq, Debug, FromPrimitive, ToPrimitive)]
pub enum ObjType {
    NullObj     = 0,
    Untyped     = 1,
    CNode       = 2,
    Tcb         = 3,
    Ram         = 4,
    VTable      = 5,
    Endpoint    = 6,
    Monitor     = 7,
    Interrupt   = 8,
}

impl Default for ObjType { fn default() -> Self { Self::NullObj } }

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