use core::convert::{From, TryFrom};

use num_traits::FromPrimitive;
use crate::error::{SysError, SysErrno, SysResult};
use crate::ipc::IpcMessageType;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
pub enum SyscallOp {
    NullSyscall = 0,
    DebugPrint,
    CapIdentify,
    Derive,
    CapCopy,
    Retype,
    TcbConfigure,
    TcbResume,
    TcbSetRegisters,
    EndpointMint,
    EndpointSend,
    EndpointRecv,
    EndpointCall,
    EndpointReply,
    EndpointReplyRecv,
    RamMap,
    VTableMap,
    MonitorMintUntyped,
    MonitorInsertTcbToCpu,
    InterruptAttachIrq,
}

#[derive(Clone, Copy, Debug)]
pub struct MsgInfo {
    pub label: SyscallOp,
    pub msglen: usize,
    pub cap_transfer: bool,
}

impl MsgInfo {
    pub const fn new(label: SyscallOp, msglen: usize) -> Self {
        Self { label, msglen, cap_transfer: false }
    }

    pub const fn new_ipc(label: SyscallOp, msglen: usize, cap_transfer: bool) -> Self {
        Self { label, msglen, cap_transfer }
    }

    pub fn get_label(&self) -> SyscallOp {
        self.label
    }

    pub const fn get_length(&self) -> usize {
        self.msglen
    }
}

/// MsgInfo layout
/// -----------------------------------------------
/// |  label  |msglen|C|                          |
/// |    8    |  4   |1|                          |
/// -----------------------------------------------
/// C: Cap transfer
/// 
impl From<MsgInfo> for usize {
    fn from(info: MsgInfo) -> Self {
        (info.label as usize) << 56
        | (info.msglen as usize) << 52
        | (info.cap_transfer as usize) << 51
    }
}

impl TryFrom<usize> for MsgInfo {
    type Error = SysError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let label = SyscallOp::from_usize(value >> 56).ok_or(SysError::InvalidValue)?;
        let msglen = (value >> 52) & 0b1111;
        let cap_transfer = ((value >> 51) & 0b1) == 1;

        Ok(Self { label, msglen, cap_transfer })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RespInfo{
    pub msgtype: IpcMessageType,
    pub msglen: usize,
    pub cap_transfer: bool,
    pub need_reply: bool,
    pub badged: bool,
    pub errno: SysErrno,
}

impl RespInfo {
    pub const fn ipc_resp(err: SysError, msglen: usize, cap_transfer: bool,
                          need_reply: bool, badged: bool) -> Self {
        Self { msgtype: IpcMessageType::Message, msglen,
               cap_transfer, need_reply, badged, errno: err.errno() }
    }

    pub const fn new_syscall_resp(err: SysError, length: usize) -> Self {
        Self {
            msgtype: IpcMessageType::Message,
            msglen: length,
            cap_transfer: false,
            need_reply: false,
            badged: false,
            errno: err.errno()
        }
    }

    pub const fn new_notification() -> Self {
        Self {
            msgtype: IpcMessageType::Notification,
            msglen: 1,
            cap_transfer: false,
            need_reply: false,
            badged: false,
            errno: SysErrno::OK
        }
    }

    pub const fn get_length(&self) -> usize {
        self.msglen
    }
}

/// MsgInfo layout
/// -----------------------------------------------
/// |type|msglen|C|R|B| errno |                   |
/// |  2 |  4   |1|1|1|   6   |                   |
/// -----------------------------------------------
/// C: Cap transfer
/// R: Need Reply
/// B: Badged
/// 
impl From<RespInfo> for usize {
    fn from(info: RespInfo) -> Self {
        (info.msgtype as usize) << 62
        | (info.msglen as usize) << 58
        | (info.cap_transfer as usize) << 57
        | (info.need_reply as usize) << 56
        | (info.badged as usize) << 55
        | (info.errno as usize) << 49
    }
}

impl TryFrom<usize> for RespInfo {
    type Error = SysError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let msgtype = IpcMessageType::from_usize(value >> 62).ok_or(SysError::InvalidValue)?;
        let msglen = (value >> 58) & 0b1111;
        let cap_transfer = (value >> 57) & 0b1 == 1;
        let need_reply = (value >> 56) & 0b1 == 1;
        let badged = (value >> 55) & 0b1 == 1;
        let errno = SysErrno::from_usize((value >> 49) & 0b111111).ok_or(SysError::InvalidValue)?;

        Ok(Self {msgtype, msglen, cap_transfer, need_reply, badged, errno})
    }
}

pub fn syscall(msg_info: MsgInfo, args: &mut [usize;6]) -> SysResult<(RespInfo, &mut [usize], usize)> {
    let ret: usize;
    let badge: usize;
    let info : usize = msg_info.into();

    unsafe { llvm_asm! {"svc 1"
        : "={x0}"(badge), "={x1}"(args[0]), "={x2}"(args[1]), "={x3}"(args[2]),
          "={x4}"(args[3]), "={x5}"(args[4]), "={x6}"(ret)
        : "{x0}"(args[0]), "{x1}"(args[1]), "{x2}"(args[2]),
          "{x3}"(args[3]), "{x4}"(args[4]), "{x5}"(args[5]), "{x6}"(info)
        : "memory"
        : "volatile"
        }
    };

    let retinfo = RespInfo::try_from(ret).unwrap();
    match retinfo.errno {
        SysErrno::OK => {
            let retlen = retinfo.get_length();
            Ok((retinfo, &mut args[..retlen], badge))
        },
        SysErrno::CSpaceNotFound => { Err(SysError::CSpaceNotFound) },
        SysErrno::CapabilityTypeError => { Err(SysError::CapabilityTypeError) },
        SysErrno::LookupError => { Err(SysError::LookupError) },
        SysErrno::UnableToDerive => { Err(SysError::UnableToDerive) },
        SysErrno::SlotNotEmpty => { Err(SysError::SlotNotEmpty) },
        SysErrno::VSpaceCapMapped => { Err(SysError::VSpaceCapMapped) },
        SysErrno::UnsupportedSyscallOp => { Err(SysError::UnsupportedSyscallOp) },
        SysErrno::VSpaceTableMiss => { Err(SysError::VSpaceTableMiss { level: args[0] as u8 }) },
        SysErrno::VSpaceSlotOccupied => { Err(SysError::VSpaceSlotOccupied { level: args[0] as u8 }) },
        SysErrno::VSpacePermissionError => { Err(SysError::VSpacePermissionError) },
        SysErrno::InvalidValue => { Err(SysError::InvalidValue) },
        SysErrno::SizeTooSmall => { Err(SysError::SizeTooSmall) },
    }
}

pub fn nop() {
    unsafe { llvm_asm!{"nop"} }
}