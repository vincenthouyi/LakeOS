use rustyl4api::error::SysResult;
use rustyl4api::syscall::{syscall, MsgInfo, SyscallOp};

use super::{CapSlot, Capability, KernelObject, ObjType};

#[derive(Debug)]
pub struct UntypedObj {}
pub type UntypedCap = Capability<UntypedObj>;

impl KernelObject for UntypedObj {
    fn obj_type() -> ObjType {
        ObjType::Untyped
    }
}

impl Capability<UntypedObj> {
    pub fn retype(
        &self,
        objtype: ObjType,
        bit_size: usize,
        slot_start: usize,
        slot_len: usize,
    ) -> SysResult<()> {
        let info = MsgInfo::new(SyscallOp::Retype, 4);
        let mut args = [
            self.slot(),
            objtype as usize,
            bit_size,
            slot_start,
            slot_len,
            0,
        ];
        syscall(info, &mut args).map(|_| ())
    }

    pub fn retype_one<T: KernelObject>(
        &self,
        bit_sz: usize,
        slot: CapSlot,
    ) -> SysResult<Capability<T>> {
        self.retype(T::obj_type(), bit_sz, slot.slot(), 1)
            .map(|_| Capability::new(slot))
    }
}
