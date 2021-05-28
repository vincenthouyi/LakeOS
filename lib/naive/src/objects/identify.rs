use crate::objects::ObjType;
use rustyl4api::error::{SysError, SysResult};
use rustyl4api::syscall::{syscall, MsgInfo, SyscallOp};

#[derive(Debug, Clone, Copy)]
pub enum IdentifyResult {
    NullObj,
    Untyped {
        paddr: usize,
        bit_sz: u8,
        is_device: bool,
        free_offset: usize,
    },
    CNode {
        bit_sz: u8,
    },
    Tcb,
    Ram {
        bit_sz: u8,
        mapped_vaddr: usize,
        mapped_asid: usize,
        is_device: bool,
    },
    VTable {
        mapped_vaddr: usize,
        mapped_asid: usize,
        level: usize,
    },
    Endpoint,
    Reply,
    Monitor,
    Interrupt,
}

pub fn cap_identify(cap_slot: usize) -> SysResult<IdentifyResult> {
    use num_traits::FromPrimitive;

    let info = MsgInfo::new(SyscallOp::CapIdentify, 1);
    let mut args = [cap_slot, 0, 0, 0, 0, 0];
    syscall(info, &mut args)?;

    let obj_type = ObjType::from_usize(args[0]).ok_or(SysError::CapabilityTypeError)?;

    Ok(match obj_type {
        ObjType::NullObj => IdentifyResult::NullObj,
        ObjType::Untyped => IdentifyResult::Untyped {
            paddr: args[1],
            bit_sz: args[2] as u8,
            is_device: args[3] == 1,
            free_offset: args[4],
        },
        ObjType::CNode => IdentifyResult::CNode {
            bit_sz: args[1] as u8,
        },
        ObjType::Tcb => IdentifyResult::Tcb,
        ObjType::Ram => IdentifyResult::Ram {
            bit_sz: args[1] as u8,
            mapped_vaddr: args[2],
            mapped_asid: args[3],
            is_device: args[4] == 1,
        },
        ObjType::VTable => IdentifyResult::VTable {
            mapped_vaddr: args[1],
            mapped_asid: args[2],
            level: args[3],
        },
        ObjType::Endpoint => IdentifyResult::Endpoint,
        ObjType::Monitor => IdentifyResult::Monitor,
        ObjType::Interrupt => IdentifyResult::Interrupt,
        ObjType::Reply => IdentifyResult::Reply,
    })
}
