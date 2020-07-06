use crate::object::ObjType;
use crate::error::SysResult;
use crate::syscall::{MsgInfo, SyscallOp, syscall};

use super::{Capability, KernelObject};

#[derive(Debug)]
pub struct EndpointObj {}

impl KernelObject for EndpointObj {
    fn obj_type() -> ObjType { ObjType::Endpoint }
}

impl Capability<EndpointObj> {
    pub fn send(&self, message: &[usize]) -> SysResult<()> {
        let info = MsgInfo::new(SyscallOp::EndpointSend, message.len());
        let mut args = [self.slot, 0, 0, 0, 0, 0];
        let copied_len = message.len().min(5);
        args[1..copied_len + 1].copy_from_slice(&message[..copied_len]);
        let ret = syscall(info, &mut args);
        return ret.map(|_|());
    }

    pub fn receive<'a, 'b>(&'a self, buf: &'b mut [usize]) -> SysResult<&'b mut [usize]> {
        let info = MsgInfo::new(SyscallOp::EndpointRecv, 1);
        let mut args = [self.slot, 0, 0, 0, 0, 0];
        let retbuf = syscall(info, &mut args)?;
        let copied_buflen = retbuf.len().min(buf.len());
        buf[..copied_buflen].copy_from_slice(&retbuf[..copied_buflen]);
        Ok(&mut buf[..copied_buflen])
    }
}
