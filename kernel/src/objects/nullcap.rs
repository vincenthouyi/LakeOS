use super::*;
use sysapi::syscall::{SyscallOp, MsgInfo, RespInfo};

#[derive(Debug)]
pub enum NullObj {}

pub type NullCap<'a> = CapRef<'a, NullObj>;

impl<'a> CapRef<'a, NullObj> {
    pub const fn mint() -> CapRaw {
        CapRaw::new(0, 0, 0, None, None, ObjType::NullObj)
    }

    pub fn insert<T>(self, raw: CapRaw) -> CapRef<'a, T> 
        where T: KernelObject + ?Sized
    {
        debug_assert_eq!(T::obj_type(), raw.cap_type());
        self.raw.set(raw);

        CapRef {
            raw: self.raw,
            cap_type: PhantomData
        }
    }

    pub fn debug_formatter(_f: &mut core::fmt::DebugStruct, _cap: &CapRaw) {
        return;
    }

    pub fn handle_invocation(&self, info: MsgInfo, tcb: &mut TcbObj) -> SysResult<()> {
        match info.get_label() {
            SyscallOp::NullSyscall => {
                kprintln!("Getting null syscall");
                tcb.set_respinfo(RespInfo::new(SysError::OK, 0));
                Ok(())
            }
            SyscallOp::DebugPrint => {
                use core::char::from_u32;

                let msg_len = info.get_length();
                if msg_len < 1 {
                    return Err(SysError::InvalidValue);
                }
                let msg = tcb.get_mr(1) as u32;
                let c = from_u32(msg)
                            .ok_or(SysError::InvalidValue)?;
                kprint!("{}", c);
                tcb.set_respinfo(RespInfo::new(SysError::OK, 0));
                Ok(())
            }
            SyscallOp::CapIdentify => {
                tcb.set_mr(1, self.cap_type() as usize);

                tcb.set_respinfo(RespInfo::new(SysError::OK, 1));

                Ok(())
            }
            _ => { Err(SysError::UnsupportedSyscallOp) }
        }
    }
}
