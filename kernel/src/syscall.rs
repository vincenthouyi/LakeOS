use crate::prelude::*;
use crate::objects::*;

pub use sysapi::syscall::{MsgInfo, RespInfo, SyscallOp};

use core::convert::TryFrom;

fn _handle_syscall(tcb: &mut TcbObj) -> SysResult<()> {
    use num_traits::FromPrimitive;

    let msginfo = tcb.get_msginfo()?;
    match msginfo.get_label() {
        SyscallOp::NullSyscall => { Ok(()) },
        SyscallOp::DebugPrint => {
            use core::char::from_u32;

            let msg_len = msginfo.get_length();
            if msg_len < 1 {
                return Err(SysError::InvalidValue);
            }
            let msg = tcb.get_mr(1) as u32;
            let c = from_u32(msg)
                        .ok_or(SysError::InvalidValue)?;
            kprint!("{}", c);
            tcb.set_respinfo(RespInfo::new(SysError::OK, 0));
            Ok(())
        },
        SyscallOp::CapIdentify => {
            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap = cspace.lookup_slot(cap_idx)?;

            let ret_num = match cap.get().cap_type() {
                ObjType::NullObj => { NullCap::try_from(cap).unwrap().identify(tcb) },
                ObjType::Untyped => { UntypedCap::try_from(cap).unwrap().identify(tcb) },
                ObjType::CNode   => { CNodeCap::try_from(cap).unwrap().identify(tcb) },
                ObjType::Tcb     => { TcbCap::try_from(cap).unwrap().identify(tcb) },
                ObjType::Ram     => { RamCap::try_from(cap).unwrap().identify(tcb) },
                ObjType::VTable  => { VTableCap::try_from(cap).unwrap().identify(tcb) },
                ObjType::Endpoint=> { EndpointCap::try_from(cap).unwrap().identify(tcb) },
                ObjType::Monitor => { MonitorCap::try_from(cap).unwrap().identify(tcb) },
                ObjType::Interrupt => { InterruptCap::try_from(cap).unwrap().identify(tcb) },
            };

            tcb.set_respinfo(RespInfo::new(SysError::OK, ret_num));

            Ok(())
        },
        SyscallOp::Derive => {
            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            if let Ok(cap) = RamCap::try_from(cap_slot) {
                let dst_cptr = tcb.get_mr(1);

                let dst_slot = cspace.lookup_slot(dst_cptr)?;
                let dst_cap = NullCap::try_from(dst_slot)?;

                let dst_cap = dst_cap.insert::<RamObj>(cap.raw());

                dst_cap.set_mapped_vaddr_asid(0, 0);

                tcb.set_respinfo(RespInfo::new(SysError::OK, 0));
                Ok(())
            } else {
                Err(SysError::UnsupportedSyscallOp)
            }
        },
        SyscallOp::Retype => {
            if msginfo.get_length() < 4 {
                return Err(SysError::InvalidValue);
            }
            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            if let Ok(cap) = UntypedCap::try_from(cap_slot) {
                let obj_type = ObjType::from_usize(tcb.get_mr(1))
                                        .ok_or(SysError::InvalidValue)?;
                let bit_size = tcb.get_mr(2);
                let slot_start = tcb.get_mr(3);
                let _slot_len = tcb.get_mr(4); //TODO: find some way to do batch op
                let slots = &cspace[slot_start];

                cap.retype(obj_type, bit_size, core::slice::from_ref(slots))?;

                tcb.set_respinfo(RespInfo::new(SysError::OK, 0));

                Ok(())
            } else {
                Err(SysError::UnsupportedSyscallOp)
            }
        },
        SyscallOp::TcbConfigure => {
            if msginfo.get_length() < 2 {
                return Err(SysError::InvalidValue);
            }
            let cap_idx = tcb.get_mr(0);
            let host_cspace = tcb.cspace()?;
            let cap_slot = host_cspace.lookup_slot(cap_idx)?;

            TcbCap::try_from(cap_slot)
                .map_err(|_| SysError::UnsupportedSyscallOp)
                .and_then(|cap| {
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

                    cap.configure(cspace_cap, vspace_cap)?;

                    tcb.set_respinfo(RespInfo::new(SysError::OK, 0));
                    Ok(())
                })
        },
        SyscallOp::TcbSetRegisters => {
            if msginfo.get_length() < 3 {
                return Err(SysError::InvalidValue);
            }

            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;
            TcbCap::try_from(cap_slot)
                .map_err(|_| SysError::UnsupportedSyscallOp)
                .and_then(|cap| {
                    let reg_flags = tcb.get_mr(1);
                    if reg_flags & 0b1000 == 0b1000 {
                        let elr = tcb.get_mr(2);
                        cap.tf.set_elr(elr);
                    }

                    if reg_flags & 0b0100 == 0b0100 {
                        let sp = tcb.get_mr(3);
                        cap.tf.set_sp(sp);
                    }

                    tcb.set_respinfo(RespInfo::new(SysError::OK, 0));
                    Ok(())
                })
        },
        SyscallOp::TcbResume => {
            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            TcbCap::try_from(cap_slot)
                .map_err(|_| SysError::UnsupportedSyscallOp)
                .and_then(|mut cap| {
                    crate::SCHEDULER.push(&cap);
                    tcb.set_respinfo(RespInfo::new(SysError::OK, 0));
                    Ok(())
                })
        },
        SyscallOp::EndpointSend => {
            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            EndpointCap::try_from(cap_slot)
                .map_err(|_| SysError::UnsupportedSyscallOp)
                .and_then(|cap| cap.handle_send(msginfo, tcb))
        }
        SyscallOp::EndpointRecv => {
            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            EndpointCap::try_from(cap_slot)
                .map_err(|_| SysError::UnsupportedSyscallOp)
                .and_then(|cap| cap.handle_recv(msginfo, tcb))
        }
        SyscallOp::EndpointCall => {
            unimplemented!()
        }
        SyscallOp::EndpointReplyRecv => {
            unimplemented!()
        }
        SyscallOp::RamMap => {
            use crate::vspace::VSpace;

            if msginfo.get_length() < 3 {
                return Err(SysError::InvalidValue);
            }

            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            if let Ok(cap) = RamCap::try_from(cap_slot) {
                if cap.mapped_vaddr() != 0 {
                    return Err(SysError::VSpaceError)
                }

                let vspace_cap_idx = tcb.get_mr(1);
                let vaddr = tcb.get_mr(2);
                let rights = tcb.get_mr(3).into();

                let vspace_cap_slot = cspace.lookup_slot(vspace_cap_idx)?;
                let vspace = VSpace::from_pgd(&*(VTableCap::try_from(vspace_cap_slot)?));

                cap.map_page(&vspace, vaddr, rights)?;

                tcb.set_respinfo(RespInfo::new(SysError::OK, 0));
                Ok(())
            } else {
                Err(SysError::UnsupportedSyscallOp)
            }
        },
        SyscallOp::VTableMap => {
            use crate::vspace::VSpace;

            if msginfo.get_length() < 3 {
                return Err(SysError::InvalidValue);
            }

            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            if let Ok(cap) = VTableCap::try_from(cap_slot) {
                if cap.mapped_vaddr() != 0 {
                    return Err(SysError::VSpaceError)
                }

                let vspace_cap_idx = tcb.get_mr(1);
                let vaddr = tcb.get_mr(2);
                let level = tcb.get_mr(3);

                let vspace_cap_slot = cspace.lookup_slot(vspace_cap_idx)?;
                let vspace = VSpace::from_pgd(&*(VTableCap::try_from(vspace_cap_slot)?));

                cap.map_vtable(&vspace, vaddr, level)?;

                tcb.set_respinfo(RespInfo::new(SysError::OK, 0));
                Ok(())
            } else {
                Err(SysError::UnsupportedSyscallOp)
            }
        },
        SyscallOp::MonitorMintUntyped => {
            if msginfo.get_length() < 4 {
                return Err(SysError::InvalidValue);
            }
            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            if let Ok(_) = MonitorCap::try_from(cap_slot) {
                let slot = tcb.get_mr(1);
                let paddr = tcb.get_mr(2);
                let bit_size = tcb.get_mr(3);
                let is_device = tcb.get_mr(4) == 1;

                let cap = cspace.lookup_slot(slot)?;

                NullCap::try_from(cap)?
                    .insert::<UntypedObj>(UntypedCap::mint(paddr, bit_size, is_device));

                tcb.set_respinfo(RespInfo::new(SysError::OK, 0));

                Ok(())
            } else {
                Err(SysError::UnsupportedSyscallOp)
            }
        },
        SyscallOp::InterruptAttachIrq => {
            let cap_idx = tcb.get_mr(0);
            let cspace = tcb.cspace()?;
            let cap_slot = cspace.lookup_slot(cap_idx)?;

            if let Ok(_cap) = InterruptCap::try_from(cap_slot) {
                use core::cell::Cell;

                let ep = tcb.get_mr(1);
                let irq = tcb.get_mr(2);

                let ep_slot = cspace.lookup_slot(ep)?;
                let ep_cap = EndpointCap::try_from(ep_slot)?;

                ep_cap.set_attach(crate::objects::Attach::Irq(irq));

                unsafe {
                    crate::interrupt::INTERRUPT_CONTROLLER.lock().attach_irq(irq, Cell::new(ep_cap.raw()));
                }

                tcb.set_respinfo(RespInfo::new(SysError::OK, 0));

                Ok(())
            } else {
                Err(SysError::UnsupportedSyscallOp)
            }
        }
    }
}

pub fn handle_syscall(tcb: &mut TcbObj) -> ! {
    let ret = _handle_syscall(tcb);

    if let Err(e) = ret {
        tcb.set_respinfo(RespInfo::new(e, 0));
    }

    crate::SCHEDULER.activate()
}
