use core::fmt::{Debug, Formatter, Error};
use crate::prelude::*;
use crate::syscall::{MsgInfo, RespInfo};
use crate::objects::TcbObj;

const EL1h: usize = 0b0101;
const EL0t: usize = 0b0000;
const AARCH64: usize = 0b0 << 4;
const FIRQ_MASK: usize = 0b1 << 6;

#[repr(C)]
#[derive(Default, Clone)]
pub struct TrapFrame {
    x_regs: [usize; 31],
    sp: usize,
    elr: usize,
    spsr: usize,
}
impl Debug for TrapFrame {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("Trapframe")
         .field("x0", &self.x_regs[0])
         .field("x1", &self.x_regs[1])
         .field("x2", &self.x_regs[2])
         .field("x3", &self.x_regs[3])
         .field("x4", &self.x_regs[4])
         .field("x5", &self.x_regs[5])
         .field("x6", &self.x_regs[6])
         .field("x7", &self.x_regs[7])
         .field("x8", &self.x_regs[8])
         .field("x9", &self.x_regs[9])
         .field("x10", &self.x_regs[10])
         .field("x11", &self.x_regs[11])
         .field("x12", &self.x_regs[12])
         .field("x13", &self.x_regs[13])
         .field("x14", &self.x_regs[14])
         .field("x15", &self.x_regs[15])
         .field("x16", &self.x_regs[16])
         .field("x17", &self.x_regs[17])
         .field("x18", &self.x_regs[18])
         .field("x19", &self.x_regs[19])
         .field("x20", &self.x_regs[20])
         .field("x21", &self.x_regs[21])
         .field("x22", &self.x_regs[22])
         .field("x23", &self.x_regs[23])
         .field("x24", &self.x_regs[24])
         .field("x25", &self.x_regs[25])
         .field("x26", &self.x_regs[26])
         .field("x27", &self.x_regs[27])
         .field("x28", &self.x_regs[28])
         .field("x29", &self.x_regs[29])
         .field("sp", &self.sp)
         .field("elr", &self.elr)
         .field("spsr", &self.spsr)
         .finish()
    }
}

impl TrapFrame {
    pub const fn new() -> Self {
        Self {
            x_regs: [0; 31],
            sp: 0,
            elr: 0,
            spsr: 0,
        }
    }

    pub unsafe fn restore(&mut self) -> ! {
        llvm_asm!{
            "
            mov     sp, $0
            ldp     x22, x23, [sp, #16 * 16]
            ldp     x30, x21, [sp, #16 * 15]
            msr     spsr_el1, x23
            msr     elr_el1, x22
            msr     sp_el0, x21
            ldp     x28, x29, [sp, #16 * 14]
            ldp     x26, x27, [sp, #16 * 13]
            ldp     x24, x25, [sp, #16 * 12]
            ldp     x22, x23, [sp, #16 * 11]
            ldp     x20, x21, [sp, #16 * 10]
            ldp     x18, x19, [sp, #16 * 9 ]
            ldp     x16, x17, [sp, #16 * 8 ]
            ldp     x14, x15, [sp, #16 * 7 ]
            ldp     x12, x13, [sp, #16 * 6 ]
            ldp     x10, x11, [sp, #16 * 5 ]
            ldp     x8,  x9,  [sp, #16 * 4 ]
            ldp     x6,  x7,  [sp, #16 * 3 ]
            ldp     x4,  x5,  [sp, #16 * 2 ]
            ldp     x2,  x3,  [sp, #16 * 1 ]
            ldp     x0,  x1,  [sp, #16 * 0 ]
            eret
            "
            ::"r"(self):"memory": "volatile"
        }

        unreachable!();
    }

    pub fn configure_idle_thread(&mut self) {
        self.set_spsr(FIRQ_MASK | AARCH64 | EL1h);
        self.set_elr(super::idle::idle_thread as usize);
    }

    pub fn init_user_thread(&mut self) {
        self.set_spsr(FIRQ_MASK | AARCH64 | EL0t);
    }

    pub fn get_elr(&self) -> usize {
        self.elr
    }

    pub fn set_elr(&mut self, elr: usize) {
        self.elr = elr;
    }

    pub fn set_spsr(&mut self, spsr: usize) {
        self.spsr = spsr;
    }

    pub fn set_sp(&mut self, sp: usize) {
        self.sp = sp;
    }

    pub fn get_mr(&self, idx: usize) -> usize {
        self.x_regs[idx]
    }

    pub fn set_mr(&mut self, idx: usize, mr: usize) {
        self.x_regs[idx] = mr;
    }

    pub fn get_msginfo(&self) -> SysResult<MsgInfo> {
        MsgInfo::try_from(self.x_regs[6])
    }

    pub fn set_respinfo(&mut self, respinfo: RespInfo) {
        self.x_regs[6] = respinfo.into();
    }

    pub fn get_tcb(&mut self) -> &mut TcbObj {
        let ptr = self as *mut _ as usize;
        unsafe{ &mut *((ptr & !MASK!(crate::objects::TCB_OBJ_BIT_SZ)) as *mut TcbObj) }
    }
}
