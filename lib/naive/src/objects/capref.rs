use super::{
    CNodeObj, CapSlot, Capability, EndpointObj, InterruptObj, KernelObject, MonitorObj, RamObj,
    ReplyObj, TcbObj, UntypedObj, VTableObj,
};
use alloc::sync::Arc;
use core::convert::From;
use core::ops::Deref;

#[derive(Clone, Debug)]
pub struct CapRef<T: KernelObject>(Arc<Capability<T>>);

impl<T: KernelObject> CapRef<T> {
    pub fn from_inner(inner: Capability<T>) -> CapRef<T> {
        Self(Arc::new(inner))
    }

    pub fn from_slot(slot: CapSlot) -> CapRef<T> {
        Self::from_inner(Capability::new(slot))
    }

    pub fn from_slot_num(slot: usize) -> CapRef<T> {
        Self::from_slot(CapSlot::new(slot))
    }
}

impl<T: KernelObject> Deref for CapRef<T> {
    type Target = Capability<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: KernelObject> From<CapSlot> for CapRef<T> {
    fn from(slot: CapSlot) -> Self {
        Self::from_slot(slot)
    }
}

impl<T: KernelObject> From<Capability<T>> for CapRef<T> {
    fn from(inner: Capability<T>) -> Self {
        Self::from_inner(inner)
    }
}

pub type MonitorRef = CapRef<MonitorObj>;
pub type UntypedRef = CapRef<UntypedObj>;
pub type VTableRef = CapRef<VTableObj>;
pub type CNodeRef = CapRef<CNodeObj>;
pub type ReplyRef = CapRef<ReplyObj>;
pub type IrqRef = CapRef<InterruptObj>;
pub type RamRef = CapRef<RamObj>;
pub type TcbRef = CapRef<TcbObj>;
pub type EpRef = CapRef<EndpointObj>;
