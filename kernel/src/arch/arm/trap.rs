use crate::console::kprintln;
use crate::arch;
use crate::objects::TcbObj;
use super::trapframe::TrapFrame;
use crate::interrupt::INTERRUPT_CONTROLLER;
use super::affinity;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Fault {
    AddressSize,
    Translation,
    AccessFlag,
    Permission,
    Alignment,
    TlbConflict,
    Other(u8)
}

impl From<u32> for Fault {
    fn from(val: u32) -> Fault {
        use self::Fault::*;

        match val & 0b111100 {
            0b000000 => AddressSize,
            0b000100 => Translation,
            0b001000 => AccessFlag,
            0b001100 => Permission,
            0b100000 => Alignment,
            0b110000 => TlbConflict,
            _ => Other((val & 0b111111) as u8),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Syndrome {
    Unknown,
    WfiWfe,
    McrMrc,
    McrrMrrc,
    LdcStc,
    SimdFp,
    Vmrs,
    Mrrc,
    IllegalExecutionState,
    Svc(u16),
    Hvc(u16),
    Smc(u16),
    MsrMrsSystem,
    InstructionAbort {
        kind: Fault,
        level: u8,
    },
    PCAlignmentFault,
    DataAbort {
        kind: Fault,
        level: u8
    },
    SpAlignmentFault,
    TrappedFpu,
    SError,
    Breakpoint,
    Step,
    Watchpoint,
    Brk(u16),
    Other(u32)
}

/// Converts a raw syndrome value (ESR) into a `Syndrome` (ref: D1.10.4).
impl From<u32> for Syndrome {
    fn from(esr: u32) -> Syndrome {
        use self::Syndrome::*;
        let iss = esr & 0xFFFFFF;

        match esr >> 26 {
            0b000000 => Unknown,
            0b000001 => WfiWfe,
            0b000011 => McrMrc,
            0b000100 => McrrMrrc,
            0b000101 => McrMrc, // How to represent diff with 0b000011?
            0b000110 => LdcStc,
            0b000111 => SimdFp,
            0b001000 => Vmrs,
            0b001100 => Mrrc,
            0b001110 => IllegalExecutionState,
            0b010001 => Svc((iss & 0xFFFF) as u16), // aarch32
            0b010010 => Hvc((iss & 0xFFFF) as u16), // aarch32
            0b010011 => Smc((iss & 0xFFFF) as u16), // aarch32
            0b010101 => Svc((iss & 0xFFFF) as u16), // aarch64
            0b010110 => Hvc((iss & 0xFFFF) as u16), // aarch64
            0b010111 => Smc((iss & 0xFFFF) as u16), // aarch64
            0b011000 => MsrMrsSystem,
            0b100000 => InstructionAbort{kind: iss.into(), level: (iss & 0b11) as u8}, // Instruction Abort from lower EL
            0b100001 => InstructionAbort{kind: iss.into(), level: (iss & 0b11) as u8}, // Instruction Abort from same EL
            0b100010 => PCAlignmentFault,
            0b100100 => DataAbort{kind: iss.into(), level: (iss & 0b11) as u8}, // from lower EL
            0b100101 => DataAbort{kind: iss.into(), level: (iss & 0b11) as u8}, // from same EL
            0b100110 => SpAlignmentFault,
            0b101000 => TrappedFpu,
            0b101100 => TrappedFpu, //diff with 0b101000?
            0b101111 => SError,
            0b110000 => Breakpoint,
            0b110001 => Breakpoint,
            0b110010 => Step,
            0b110011 => Step,
            0b110100 => Watchpoint,
            0b110101 => Watchpoint,
            0b111000 => Breakpoint,
            0b111100 => Brk((iss & 0xFFFF) as u16),
            _ => Other(esr),
        
        }
    }
}


//#[repr(u16)]
//#[derive(Debug, PartialEq, Eq, Copy, Clone)]
//pub enum Kind {
//    Synchronous = 0,
//    Irq = 1,
//    Fiq = 2,
//    SError = 3,
//}
//
//#[repr(u16)]
//#[derive(Debug, PartialEq, Eq, Copy, Clone)]
//pub enum Source {
//    CurrentSpEl0 = 0,
//    CurrentSpElx = 1,
//    LowerAArch64 = 2,
//    LowerAArch32 = 3,
//}
//
//#[repr(C)]
//#[derive(Debug, PartialEq, Eq, Copy, Clone)]
//pub struct Info {
//    source: Source,
//    kind: Kind,
//}

//#[no_mangle]
//pub unsafe extern "C" fn trap_handler(info: Info, tcb: &mut TcbObj) {
//
//    let addr = arch::get_far();
//    let esr = arch::get_esr();
//    let elr = arch::get_elr();
//
////    kprintln!("in trap! {:?} {:?} esr: {:x} elr: {:x} addr {:x}",
////               info, Syndrome::from(esr), esr, tcb.tf.elr, addr);
////    kprintln!("tcb {:p} sp {:?}", &tcb, arch::sp());
////    kprintln!("tcb {:?}", &tcb);
//
//    if let (Kind::Synchronous, Syndrome::Brk(_)) = (info.kind, Syndrome::from(esr)) {
//        tcb.tf.elr += 4;
//    }
//
//    tcb.restore();
//}

pub fn handle_vfault(tcb: &mut TcbObj) -> ! {
    let fault_addr = arch::get_far();

    kprintln!("thread {:x} faulting addr 0x{:x} elr {:x}", tcb.thread_id() ,fault_addr, tcb.tf.get_elr());

    let vspace = tcb.vspace().unwrap();
    let slot = vspace.lookup_pt_slot(fault_addr);
    kprintln!("slot {:x?}", slot.unwrap());

    unimplemented!("unable to handle vfault!");
}

#[no_mangle]
pub unsafe extern "C" fn sync_handler() -> ! {
    use self::Syndrome::*;

    kprintln!("Panic! kernel hitting exception!");

    match Syndrome::from(arch::get_esr()) {
        InstructionAbort {kind,level} => {
            kprintln!("Instruction Abort: kind {:?} level {} syndrome {:?} elr {:x}", kind, level, Syndrome::from(arch::get_esr()), arch::get_elr());
        },
        DataAbort {kind,level} => {
            kprintln!("Data Abort: kind {:?} level {} syndrome {:?} elr 0x{:x} fault address 0x{:x}", kind, level, Syndrome::from(arch::get_esr()), arch::get_elr(), arch::get_far());
        },
        syn => {
            kprintln!("Unhandled synchronous trap: {:?}", syn);
        }
    }

    panic!("Unable to handle kernel space exception!");
}

#[no_mangle]
pub unsafe extern "C" fn lower64_sync_handler(tf: &mut TrapFrame) -> ! {
    use self::Syndrome::*;

    let tcb = tf.get_tcb();
    let _ret = match Syndrome::from(arch::get_esr()) {
        Svc(1) => {
            crate::syscall::handle_syscall(tcb)
        },
        InstructionAbort {kind,level} => {
            kprintln!("Lower64 Instruction Abort tcb {:X?} kind {:?} level {} syndrome {:?}", tcb, kind, level, Syndrome::from(arch::get_esr()));
            handle_vfault(tcb)
        },
        DataAbort {kind,level} => {
            kprintln!("Lower64 Data Abort tcb {:X?} kind {:?} level {} syndrome {:?}", tcb, kind, level, Syndrome::from(arch::get_esr()));
            handle_vfault(tcb)
        },
        Brk(_) => {
            kprintln!("tcb {:X?} syndrome {:?}", tcb, Syndrome::from(arch::get_esr()));
            let elr = tcb.tf.get_elr();
            tcb.tf.set_elr(elr+4);
            unreachable!()
        },
        syn => {
            panic!("Unhandled synchronous trap: {:?} thread_id {:x} elr {:x} tcb {:x?}", syn, tcb.thread_id(), tf.get_elr(), tf);
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn lower64_irq_handler(tf: &mut TrapFrame) -> ! {
    use super::generic_timer::Timer;

    let cpuid = affinity();
    let tcb = tf.get_tcb();
    let mut timer = Timer::new();
    if timer.is_pending(cpuid) {
        tcb.timeslice_sub(crate::TICK as usize);
        timer.tick_in(crate::TICK);
    } else {
        INTERRUPT_CONTROLLER.lock().receive_irq();
    }

    crate::SCHEDULER.get().activate();
}

#[no_mangle]
pub unsafe extern "C" fn unknown_exception_handler(tcb: &mut TcbObj) -> ! {
    kprintln!("unknown exception! tcb: {:?}", tcb);
    loop{}
}

#[no_mangle]
pub unsafe extern "C" fn irq_trap() -> ! {
    use super::generic_timer::Timer;

    // INTERRUPT_CONTROLLER.lock().receive_irq();
    super::boot::IDLE_THREADS.get_mut().timeslice_sub(crate::TICK as usize);
    Timer::new().tick_in(crate::TICK);
    crate::SCHEDULER.get_mut().activate();
}
