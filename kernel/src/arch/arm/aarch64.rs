/// Returns the current stack pointer.
#[allow(dead_code)]
#[inline(always)]
pub fn sp() -> *const u8 {
    let ptr: usize;
    unsafe {
        llvm_asm!("mov $0, sp" : "=r"(ptr));
    }

    ptr as *const u8
}

#[allow(dead_code)]
#[inline(always)]
pub fn mpidr_el1() -> usize {
    let x: usize;
    unsafe{
        llvm_asm!("mrs     $0, mpidr_el1"
            : "=r"(x));
    }
    x
}

#[allow(dead_code)]
#[inline(always)]
pub fn tpidr_el1() -> usize {
    let x: usize;
    unsafe{
        llvm_asm!("mrs     $0, tpidr_el1"
            : "=r"(x));
    }
    x
}

/// Returns the current exception level.
///
/// # Safety
/// This function should only be called when EL is >= 1.
//#[inline(always)]
//pub fn current_el() -> u8 {
//    let el_reg: u64;
//    unsafe{
//        llvm_asm!("mrs $0, CurrentEL" : "=r"(el_reg));
//    }
//    ((el_reg & 0b1100) >> 2) as u8
//}

///// Returns the SPSel value.
//#[inline(always)]
//pub fn sp_sel() -> u8 {
//    let ptr: u32;
//    unsafe {
//        llvm_asm!("mrs $0, SPSel" : "=r"(ptr));
//    }
//
//    (ptr & 1) as u8
//}

/// Returns the core currently executing.
///
/// # Safety
///
/// This function should only be called when EL is >= 1.
#[allow(dead_code)]
#[inline(always)]
pub fn affinity() -> usize {
    mpidr_el1() & 0x3
}

///// A NOOP that won't be optimized out.
//#[inline(always)]
//pub fn nop() {
//    unsafe {
//        llvm_asm!("nop" :::: "volatile");
//    }
//}

#[allow(dead_code)]
#[inline(always)]
pub fn wfe() {
    unsafe {
        llvm_asm!("wfe");
    }
}

#[allow(dead_code)]
#[inline(always)]
pub fn wfi() {
    unsafe {
        llvm_asm!("wfi");
    }
}

#[inline(always)]
pub fn isb() {
    unsafe{ llvm_asm!("isb" ::: "memory") }
}

#[inline(always)]
pub fn dsb() {
    unsafe{ llvm_asm!("dsb sy" ::: "memory" : "volatile") }
}

#[inline(always)]
pub fn dmb() {
    unsafe{ llvm_asm!("dmb sy" ::: "memory") }
}

#[inline(always)]
pub fn get_elr() -> usize
{
    let elr;
    unsafe {
        llvm_asm!("mrs $0, elr_el1":"=r"(elr));
    }
    elr
}

#[inline(always)]
pub fn get_esr() -> u32
{
    let esr;
    unsafe {
        llvm_asm!("mrs $0, esr_el1":"=r"(esr));
    }
    esr
}

#[inline(always)]
pub fn get_far() -> usize
{
    let far;
    unsafe {
        llvm_asm!("mrs $0, far_el1":"=r"(far));
    }
    far
}

//pub fn dc_clean_by_va_PoU(vaddr: usize)
//{
//    unsafe { llvm_asm!("dc cvau, $0": : "r"(vaddr): :"volatile") }
//    dmb();
//}

pub fn dc_clean_by_va_PoC(vaddr: usize)
{
    unsafe { llvm_asm!("dc cvac, $0": : "r"(vaddr): :"volatile") }
    dmb();
}

#[allow(dead_code)]
#[inline(always)]
pub fn spsr_el1() -> usize {
    let x;
    unsafe { llvm_asm!("mrs $0, spsr_el1":"=r"(x)); }
    x
}