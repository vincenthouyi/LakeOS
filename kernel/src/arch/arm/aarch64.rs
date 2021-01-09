use aarch64::{
    barrier::{dsb as dsb_impl, isb as isb_impl, SY},
    cache::{Cache, Clean, DCache, ICache, PoU},
    regs::{RegisterReadWrite, ESR_EL1, FAR_EL1},
};

pub use aarch64::asm::{cpuid, nop, sp, wfe, wfi};

#[inline(always)]
pub fn isb() {
    unsafe { isb_impl() }
}

#[inline(always)]
pub fn dsb() {
    unsafe { dsb_impl(SY) }
}

#[inline(always)]
pub fn get_elr() -> usize {
    let elr;
    unsafe {
        llvm_asm!("mrs $0, elr_el1":"=r"(elr));
    }
    elr
}

#[inline(always)]
pub fn get_esr() -> u32 {
    ESR_EL1.get() as u32
}

#[inline(always)]
pub fn get_far() -> u64 {
    FAR_EL1.get()
}

pub fn dc_clean_by_va_PoU(vaddr: usize) {
    DCache::<Clean, PoU>::flush_line_op(vaddr);
    dsb();
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
        if cache_type(clid, l) > 0b001 {
            // ICache
            let s = read_cache_size(l, false);
            let line_bits = (s & MASK!(3)) + 4;
            let assoc = ((s >> 3) & MASK!(10)) + 1;
            let assoc_bits = 64 - (assoc - 1).leading_zeros();
            let nsets = ((s >> 13) & MASK!(15)) + 1;

            for w in 0..assoc {
                for s in 0..nsets {
                    let wsl = w << (32 - assoc_bits) | (s << line_bits) | (l << 1);
                    unsafe { llvm_asm!("dc cisw, $0"::"r"(wsl)) }
                }
            }
        }
    }
}

pub fn clean_l1_cache() {
    dsb();
    clean_dcache_poc();
    dsb();
    ICache::local_flush_all();
    dsb();
}
