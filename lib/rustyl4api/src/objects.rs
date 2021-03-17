
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

pub const TCB_OBJ_SZ: usize = 1024;
pub const TCB_OBJ_BIT_SZ: usize = 10;

pub const CNODE_DEPTH: usize = core::mem::size_of::<usize>() * 8;
pub const CNODE_ENTRY_BIT_SZ: usize = 6;
pub const CNODE_ENTRY_SZ: usize = 1 << CNODE_ENTRY_BIT_SZ;