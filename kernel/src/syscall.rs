use crate::prelude::*;
use crate::objects::*;

pub use sysapi::syscall::{MsgInfo, RespInfo, SyscallOp};

use core::convert::TryFrom;

fn _handle_syscall(tcb: &mut TcbObj) -> SysResult<()> {
    use num_traits::FromPrimitive;

    let tcb2 = unsafe{ &mut *(tcb as *mut TcbObj) };
    let msginfo = MsgInfo::from_usize(tcb.tf.x_regs[6]).unwrap();
    let cspace = tcb2.cspace().unwrap();
    let cap_idx = tcb.get_mr(0);
    let cap = cspace.lookup_slot(cap_idx)?;

//    kprintln!("syscall {:?} cap type {:?}", msginfo.get_label(), cap.get().cap_type());
    let ret = match cap.get().cap_type() {
        ObjType::NullObj => { NullCap::try_from(cap).unwrap().handle_invocation(msginfo, tcb) },
        ObjType::Untyped => { UntypedCap::try_from(cap).unwrap().handle_invocation(msginfo, tcb) },
        ObjType::CNode   => { CNodeCap::try_from(cap).unwrap().handle_invocation(msginfo, tcb) },
        ObjType::Tcb     => { TcbCap::try_from(cap).unwrap().handle_invocation(msginfo, tcb) },
        ObjType::Ram     => { RamCap::try_from(cap).unwrap().handle_invocation(msginfo, tcb) },
        ObjType::VTable  => { VTableCap::try_from(cap).unwrap().handle_invocation(msginfo, tcb) },
        ObjType::Endpoint=> { EndpointCap::try_from(cap).unwrap().handle_invocation(msginfo, tcb) },
        ObjType::Monitor => { MonitorCap::try_from(cap).unwrap().handle_invocation(msginfo, tcb) },
        ObjType::Interrupt => { InterruptCap::try_from(cap).unwrap().handle_invocation(msginfo, tcb) },
    };

    if let Err(e) = ret {
       kprintln!("cap_idx {} type {:?} syscall {:?} cap type {:?} err {:?}",
                   cap_idx, cap.get().cap_type(), msginfo.get_label(), cap.get().cap_type(), e);
    }

    ret
}

pub fn handle_syscall(tcb: &mut TcbObj) -> ! {
    let ret = _handle_syscall(tcb);

    if let Err(e) = ret {
        tcb.set_respinfo(RespInfo::new(e, 0));
    }

    crate::SCHEDULER.activate()
}
