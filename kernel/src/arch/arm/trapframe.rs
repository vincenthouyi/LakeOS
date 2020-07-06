use core::cell::Cell;
use crate::prelude::*;
use crate::syscall::{MsgInfo, RespInfo};
use num_traits::FromPrimitive;
use crate::objects::TcbObj;

#[repr(C)]
#[derive(Default, Clone, Debug)]
pub struct TrapFrame {
    x_regs: [Cell<usize>; 31],
    sp: Cell<usize>,
    elr: Cell<usize>,
    spsr: Cell<usize>,
}

impl TrapFrame {
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

    pub fn get_elr(&self) -> usize {
        self.elr.get()
    }

    pub fn set_elr(&self, elr: usize) {
        self.elr.set(elr);
    }

    pub fn set_spsr(&self, spsr: usize) {
        self.spsr.set(spsr);
    }

    pub fn set_sp(&self, sp: usize) {
        self.sp.set(sp);
    }

    pub fn get_mr(&self, idx: usize) -> usize {
        self.x_regs[idx].get()
    }

    pub fn set_mr(&self, idx: usize, mr: usize) {
        self.x_regs[idx].set(mr);
    }

    pub fn get_msginfo(&self) -> SysResult<MsgInfo> {
        MsgInfo::from_usize(self.x_regs[6].get()).ok_or(SysError::InvalidValue)
    }

    pub fn set_respinfo(&self, respinfo: RespInfo) {
        self.x_regs[6].set(respinfo.0);
    }

    pub fn get_tcb(&mut self) -> &mut TcbObj {
        let ptr = self as *mut _ as usize;
        unsafe{ &mut *((ptr & !MASK!(crate::objects::TCB_OBJ_BIT_SZ)) as *mut TcbObj) }
    }
}
