use core::num::NonZeroUsize;

use crate::objects::*;
use crate::prelude::*;

pub use sysapi::syscall::{MsgInfo, RespInfo, SyscallOp};

use core::convert::TryFrom;

fn _handle_syscall(tcb: &mut TcbObj) -> SysResult<()> {
    use num_traits::FromPrimitive;

    let msginfo = tcb.get_msginfo()?;
    match msginfo.get_label() {
        SyscallOp::NullSyscall => Ok(()),
        SyscallOp::DebugPrint => {
            use core::char::from_u32;

            let msg_len = msginfo.get_length();
            if msg_len < 1 {
                return Err(SysError::InvalidValue);
            }
            let msg = tcb.get_mr(1) as u32;
            let c = from_u32(msg).ok_or(SysError::InvalidValue)?;
            kprint!("{}", c);
            tcb.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, 0));
            Ok(())
        }
        SyscallOp::CapIdentify => {
            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap = cspace.lookup_slot(cap_idx)?;

            let ret_num = match cap.get().cap_type() {
                ObjType::NullObj => NullCap::try_from(cap).unwrap().identify(tcb),
                ObjType::Untyped => UntypedCap::try_from(cap).unwrap().identify(tcb),
                ObjType::CNode => CNodeCap::try_from(cap).unwrap().identify(tcb),
                ObjType::Tcb => TcbCap::try_from(cap).unwrap().identify(tcb),
                ObjType::Ram => RamCap::try_from(cap).unwrap().identify(tcb),
                ObjType::VTable => VTableCap::try_from(cap).unwrap().identify(tcb),
                ObjType::Endpoint => EndpointCap::try_from(cap).unwrap().identify(tcb),
                ObjType::Reply => ReplyCap::try_from(cap).unwrap().identify(tcb),
                ObjType::Monitor => MonitorCap::try_from(cap).unwrap().identify(tcb),
                ObjType::Interrupt => InterruptCap::try_from(cap).unwrap().identify(tcb),
            };

            tcb.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, ret_num));

            Ok(())
        }
        SyscallOp::CapCopy => {
            let cspace = tcb.cspace()?;

            let dst_croot_cptr = tcb.get_mr(0);
            let dst_croot_slot = cspace.lookup_slot(dst_croot_cptr)?;
            let dst_croot_cap = CNodeCap::try_from(dst_croot_slot)?;

            let dst_offset = tcb.get_mr(1);
            let dst_slot = dst_croot_cap.lookup_slot(dst_offset)?;
            let dst_cap = NullCap::try_from(dst_slot)?;

            let cap_idx = tcb.get_mr(2);
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            let badge = tcb.get_mr(3);
            let badge = NonZeroUsize::new(badge);

            let derived_raw = cap_derive(&cap_slot, badge)?;
            dst_cap.insert_raw(derived_raw);
            cnode_entry_append_next(&cap_slot, dst_cap.raw);

            tcb.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, 0));
            Ok(())
        }
        SyscallOp::CNodeDelete => {
            let cspace = tcb.cspace()?;

            let cptr = tcb.get_mr(0);
            let cap_slot = cspace.lookup_slot(cptr)?;
            if cap_slot.get().cap_type() == ObjType::NullObj {
                return Err(SysError::CapabilityTypeError);
            }

            //TODO: check children etc.
            //TODO: changing next and prev cap ptr.
            cap_slot.set(NullCap::mint());
            Ok(())
        }
        SyscallOp::Retype => {
            if msginfo.get_length() < 4 {
                return Err(SysError::InvalidValue);
            }
            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;
            let cap = UntypedCap::try_from(cap_slot)?;

            let obj_type = ObjType::from_usize(tcb.get_mr(1)).ok_or(SysError::InvalidValue)?;
            let bit_size = tcb.get_mr(2);
            let slot_start = tcb.get_mr(3);
            let _slot_len = tcb.get_mr(4); //TODO: find some way to do batch op
            let slots = &cspace[slot_start];

            cap.retype(obj_type, bit_size, core::slice::from_ref(slots))?;

            tcb.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, 0));

            Ok(())
        }
        SyscallOp::TcbConfigure => {
            if msginfo.get_length() < 3 {
                return Err(SysError::InvalidValue);
            }
            let cap_idx = tcb.get_mr(0);
            let host_cspace = tcb.cspace()?;
            let cap_slot = host_cspace.lookup_slot(cap_idx)?;
            let cap = TcbCap::try_from(cap_slot)?;

            let vspace_cap_idx = tcb.get_mr(1);
            let vspace_cap = if vspace_cap_idx != 0 {
                let vspace_slot = host_cspace.lookup_slot(vspace_cap_idx)?;
                Some(VTableCap::try_from(vspace_slot)?)
            } else {
                None
            };

            let cspace_cap_idx = tcb.get_mr(2);
            let cspace_cap = if cspace_cap_idx != 0 {
                let cspace_slot = host_cspace.lookup_slot(cspace_cap_idx)?;
                Some(CNodeCap::try_from(cspace_slot)?)
            } else {
                None
            };

            let fault_ep_cap_idx = tcb.get_mr(3);
            let fault_ep_cap = if fault_ep_cap_idx != 0 {
                let ep_slot = host_cspace.lookup_slot(fault_ep_cap_idx)?;
                Some(EndpointCap::try_from(ep_slot)?)
            } else {
                None
            };

            cap.configure(cspace_cap, vspace_cap, fault_ep_cap)?;

            tcb.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, 0));
            Ok(())
        }
        SyscallOp::TcbSetRegisters => {
            if msginfo.get_length() < 3 {
                return Err(SysError::InvalidValue);
            }

            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;
            let mut cap = TcbCap::try_from(cap_slot)?;

            let reg_flags = tcb.get_mr(1);
            if reg_flags & 0b1000 == 0b1000 {
                let elr = tcb.get_mr(2);
                cap.tf.set_elr(elr);
            }

            if reg_flags & 0b0100 == 0b0100 {
                let sp = tcb.get_mr(3);
                cap.tf.set_sp(sp);
            }

            tcb.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, 0));
            Ok(())
        }
        SyscallOp::TcbResume => {
            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            let cap = TcbCap::try_from(cap_slot)?;
            crate::SCHEDULER.get_mut().push(&cap);

            tcb.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, 0));
            Ok(())
        }
        SyscallOp::EndpointSend => {
            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            let cap = EndpointCap::try_from(cap_slot)?;
            cap.handle_send(msginfo, tcb)?;

            Ok(())
        }
        SyscallOp::EndpointRecv => {
            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            let cap = EndpointCap::try_from(cap_slot)?;
            cap.handle_recv(msginfo, tcb)?;

            Ok(())
        }
        SyscallOp::EndpointCall => {
            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            let cap = EndpointCap::try_from(cap_slot)?;
            cap.handle_call(msginfo, tcb)?;

            Ok(())
        }
        SyscallOp::EndpointReply => {
            let reply = tcb.reply_cap().ok_or(SysError::LookupError)?;
            reply.handle_reply(msginfo, tcb, false)?;
            // tcb.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, 0));

            Ok(())
        }
        SyscallOp::EndpointReplyRecv => {
            let reply = tcb.reply_cap().ok_or(SysError::LookupError)?;
            reply.handle_reply(msginfo, tcb, true)?;

            // let cap_idx = tcb.get_mr(0);
            // let cspace = tcb.cspace()?;
            // let cap_slot = cspace.lookup_slot(cap_idx)?;

            // let cap = EndpointCap::try_from(cap_slot)?;
            // cap.handle_recv(msginfo, tcb)?;

            Ok(())
        }
        SyscallOp::RamMap => {
            use crate::vspace::VSpace;

            if msginfo.get_length() < 3 {
                return Err(SysError::InvalidValue);
            }

            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            let cap = RamCap::try_from(cap_slot)?;

            if cap.mapped_vaddr() != 0 {
                return Err(SysError::VSpaceCapMapped);
            }

            let vspace_cap_idx = tcb.get_mr(1);
            let vaddr = tcb.get_mr(2);
            let rights = tcb.get_mr(3).into();

            let vspace_cap_slot = cspace.lookup_slot(vspace_cap_idx)?;
            let vspace = VSpace::from_pgd(&*(VTableCap::try_from(vspace_cap_slot)?));

            cap.map_page(&vspace, vaddr, rights)?;

            tcb.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, 0));
            Ok(())
        }
        SyscallOp::RamUnmap => {
            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            let cap = RamCap::try_from(cap_slot)?;

            if cap.mapped_vaddr() == 0 {
                return Err(SysError::VSpaceCapNotMapped);
            }

            cap.unmap_page()?;

            tcb.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, 0));
            Ok(())
        }
        SyscallOp::VTableMap => {
            use crate::vspace::VSpace;

            if msginfo.get_length() < 3 {
                return Err(SysError::InvalidValue);
            }

            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            let cap = VTableCap::try_from(cap_slot)?;

            if cap.mapped_vaddr() != 0 {
                return Err(SysError::VSpaceCapMapped);
            }

            let vspace_cap_idx = tcb.get_mr(1);
            let vaddr = tcb.get_mr(2);
            let level = tcb.get_mr(3);

            let vspace_cap_slot = cspace.lookup_slot(vspace_cap_idx)?;
            let vspace = VSpace::from_pgd(&*(VTableCap::try_from(vspace_cap_slot)?));

            cap.map_vtable(&vspace, vaddr, level)?;

            tcb.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, 0));
            Ok(())
        }
        SyscallOp::MonitorMintUntyped => {
            if msginfo.get_length() < 4 {
                return Err(SysError::InvalidValue);
            }
            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            let _cap = MonitorCap::try_from(cap_slot)?;

            let slot = tcb.get_mr(1);
            let paddr = tcb.get_mr(2);
            let bit_size = tcb.get_mr(3);
            let is_device = tcb.get_mr(4) == 1;

            let cap = cspace.lookup_slot(slot)?;

            NullCap::try_from(cap)?
                .insert::<UntypedObj>(UntypedCap::mint(paddr, bit_size, is_device));

            tcb.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, 0));

            Ok(())
        }
        SyscallOp::MonitorInsertTcbToCpu => {
            use crate::scheduler::SCHEDULER;
            // if msginfo.get_length() < 4 {
            //     return Err(SysError::InvalidValue);
            // }
            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            MonitorCap::try_from(cap_slot)?;

            let tcb_idx = tcb.get_mr(1);
            let cpu = tcb.get_mr(2);

            let tcb_slot = cspace.lookup_slot(tcb_idx)?;

            let tcb_cap = TcbCap::try_from(tcb_slot)?;
            unsafe {
                SCHEDULER.get_unsafe(cpu).push(&tcb_cap);
            }

            tcb.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, 0));

            Ok(())
        }
        SyscallOp::InterruptAttachIrq => {
            use core::cell::Cell;

            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            InterruptCap::try_from(cap_slot)?;

            let ep = tcb.get_mr(1);
            let irq = tcb.get_mr(2);

            let ep_slot = cspace.lookup_slot(ep)?;
            let ep_cap = EndpointCap::try_from(ep_slot)?;

            ep_cap.set_attach(crate::objects::Attach::Irq(irq));

            unsafe {
                crate::interrupt::INTERRUPT_CONTROLLER
                    .lock()
                    .attach_irq(irq, Cell::new(ep_cap.raw()));
            }

            tcb.set_respinfo(RespInfo::new_syscall_resp(SysError::OK, 0));

            Ok(())
        }
    }
}

pub fn handle_syscall(tcb: &mut TcbObj) -> ! {
    if let Err(e) = _handle_syscall(tcb) {
        // kprintln!("Syscall Error {:?} info: {:?} TCB {:?}", e, tcb.get_msginfo().unwrap().get_label(), tcb);
        match e {
            SysError::VSpaceTableMiss { level } => tcb.set_mr(1, level as usize),
            SysError::VSpaceSlotOccupied { level } => tcb.set_mr(1, level as usize),
            _ => {}
        }
        tcb.set_respinfo(RespInfo::new_syscall_resp(e, 0));
    }

    crate::SCHEDULER.get_mut().activate()
}
