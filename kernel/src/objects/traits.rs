use super::*;

pub trait KernelObject {
    const obj_type: ObjType;
}

impl KernelObject for NullObj {
    const obj_type: ObjType = ObjType::NullObj;
}

impl<'a> KernelObject for CNodeObj {
    const obj_type: ObjType = ObjType::CNode;
}

impl KernelObject for UntypedObj {
    const obj_type: ObjType = ObjType::Untyped;
}

impl<'a> KernelObject for TcbObj {
    const obj_type: ObjType = ObjType::Tcb;
}

impl KernelObject for RamObj {
    const obj_type: ObjType = ObjType::Ram;
}

impl KernelObject for VTableObj {
    const obj_type: ObjType = ObjType::VTable;
}

impl KernelObject for EndpointObj {
    const obj_type: ObjType = ObjType::Endpoint;
}

impl KernelObject for ReplyObj {
    const obj_type: ObjType = ObjType::Reply;
}

impl KernelObject for MonitorObj {
    const obj_type: ObjType = ObjType::Monitor;
}

impl KernelObject for InterruptObj {
    const obj_type: ObjType = ObjType::Interrupt;
}
