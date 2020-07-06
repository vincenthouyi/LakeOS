use num_traits::FromPrimitive;
use crate::error::{SysError, SysResult};

#[repr(C)]
#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
pub enum SyscallOp {
    NullSyscall = 0,
    DebugPrint,
    CapIdentify,
    Derive,
    Retype,
    TcbConfigure,
    TcbResume,
    TcbSetRegisters,
    EndpointSend,
    EndpointRecv,
    EndpointCall,
    EndpointReplyRecv,
    RamMap,
    VTableMap,
    MonitorMintUntyped,
    InterruptAttachIrq,
}

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
pub struct MsgInfo(pub usize);

impl MsgInfo {
    const LEN_MASK: usize = 0xff;

    pub const fn new(label: SyscallOp, length: usize) -> Self {
        Self(((label as usize) << 12) | (length & Self::LEN_MASK))
    }

    pub fn get_label(&self) -> SyscallOp {
        SyscallOp::from_u64((self.0 >> 12) as u64).unwrap()
    }

    pub const fn get_length(&self) -> usize {
        self.0 & Self::LEN_MASK
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
pub struct RespInfo(pub usize);

impl RespInfo {
    const LEN_MASK: usize = 0xff;

    pub const fn _new(err: SysError, length: usize, is_notification: bool) -> Self {
        Self(
            ((err as usize) << 12) |
            ((is_notification as usize) << 8) |
            (length & Self::LEN_MASK)
        )
    }

    pub const fn new(err: SysError, length: usize) -> Self {
        Self::_new(err, length, false)
    }

    pub const fn new_notification() -> Self {
        Self::_new(SysError::OK, 1, true)
    }

    pub const fn get_length(&self) -> usize {
        self.0 & Self::LEN_MASK
    }

    pub fn as_result(&self) -> SysResult<()> {
        let err = SysError::from_usize(self.0 >> 12).unwrap();
        match err {
            SysError::OK => Ok(()),
            e => Err(e)
        }
    }
}

pub fn syscall(msg_info: MsgInfo, args: &mut [usize;6]) -> SysResult<&mut [usize]> {
    let ret: usize;

    unsafe{ llvm_asm! {"svc 1"
        : "={x1}"(args[0]), "={x2}"(args[1]), "={x3}"(args[2]),
          "={x4}"(args[3]), "={x5}"(args[4]), "={x6}"(ret)
        : "{x0}"(args[0]), "{x1}"(args[1]), "{x2}"(args[2]),
          "{x3}"(args[3]), "{x4}"(args[4]), "{x5}"(args[5]), "{x6}"(msg_info.0)
        : "memory"
        : "volatile"
    } };

    let retinfo = RespInfo::from_usize(ret).unwrap();
    retinfo.as_result()
        .map(move |_| {
            let retlen = retinfo.get_length();
            &mut args[..retlen]
        })
}

pub fn nop() {
    unsafe { llvm_asm!{"nop"} }
}