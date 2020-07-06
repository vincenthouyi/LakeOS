use super::{ObjType, KernelObject, Capability};

pub const CNODE_DEPTH: usize = core::mem::size_of::<usize>() * 8;

#[derive(Debug)]
pub enum CNodeObj { }

pub type CNodeCap = Capability<CNodeObj>;

impl KernelObject for CNodeObj {
    fn obj_type() -> ObjType { ObjType::CNode }
}