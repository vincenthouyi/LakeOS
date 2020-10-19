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

pub fn dc_clean_by_va_PoU(vaddr: usize)
{
    unsafe { llvm_asm!("dc cvau, $0": : "r"(vaddr): :"volatile") }
    dmb();
}

#[allow(dead_code)]
#[inline(always)]
pub fn spsr_el1() -> usize {
    let x;
    unsafe { llvm_asm!("mrs $0, spsr_el1":"=r"(x)); }
    x
}

pub fn clid() -> usize {
    let clid: usize;
    unsafe {
        llvm_asm!("mrs $0, clidr_el1": "=r"(clid));
    }

    return clid;
}

pub fn cache_type(clid: usize, level: usize) -> usize {
    (clid >> (level * 3)) & MASK!(3)
}

pub fn read_cache_size(level: usize, instruction: bool) -> usize {
    let cssr_old: usize;
    let size: usize;
    let selector = level < 1 | (instruction as usize);
    unsafe {
        llvm_asm!("
            mrs $0, csselr_el1
            msr csselr_el1, $1
            " : "=r"(cssr_old) : "r" (selector));
        llvm_asm!("
            mrs $0, ccsidr_el1
            msr csselr_el1, $1
        ": "=r"(size) : "r"(cssr_old))
    }

    size
}

pub fn clean_dcache_poc() {
    let clid = clid();
    let loc = (clid >> 24) & MASK!(3); // level of coherence

    for l in 0..loc {
        if cache_type(clid, l) > 0b001 { // ICache
            let s = read_cache_size(l, false);
            let line_bits = (s & MASK!(3)) + 4;
            let assoc = ((s >> 3) & MASK!(10)) + 1;
            let assoc_bits = 64 - (assoc - 1).leading_zeros();
            let nsets = ((s >> 13) & MASK!(15)) + 1;

            for w in 0..assoc {
                for s in 0..nsets {
                    let wsl = w << (32 - assoc_bits) | (s << line_bits) | (l << 1);
                    unsafe {
                        llvm_asm!("dc cisw, $0"::"r"(wsl))
                    }
                }
            }
            
        }
    }
}

pub fn clean_l1_cache() {
    dsb();
    clean_dcache_poc();
    dsb();
    unsafe { llvm_asm!("ic iallu") }
    dsb();
}