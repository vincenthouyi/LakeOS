use super::*;

pub trait KernelObject: {
    fn obj_type() -> ObjType;
}

impl KernelObject for NullObj {
    fn obj_type() -> ObjType { ObjType::NullObj }
}

impl<'a> KernelObject for CNodeObj {
    fn obj_type() -> ObjType { ObjType::CNode }
}

impl KernelObject for UntypedObj {
    fn obj_type() -> ObjType { ObjType::Untyped }
}

impl<'a> KernelObject for TcbObj {
    fn obj_type() -> ObjType { ObjType::Tcb }
}

impl KernelObject for RamObj {
    fn obj_type() -> ObjType { ObjType::Ram }
}

impl KernelObject for VTableObj {
    fn obj_type() -> ObjType { ObjType::VTable }
}

impl KernelObject for EndpointObj {
    fn obj_type() -> ObjType { ObjType::Endpoint }
}

impl KernelObject for MonitorObj {
    fn obj_type() -> ObjType { ObjType::Monitor }
}

impl KernelObject for InterruptObj {
    fn obj_type() -> ObjType { ObjType::Interrupt }
}
