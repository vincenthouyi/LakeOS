use super::{ObjType, KernelObject};

pub const CNODE_DEPTH: usize = core::mem::size_of::<usize>() * 8;

pub enum CNodeObj { }

impl KernelObject for CNodeObj {
    fn obj_type() -> ObjType { ObjType::CNode }
}