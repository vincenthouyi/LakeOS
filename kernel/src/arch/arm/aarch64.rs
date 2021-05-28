pub fn mpidr_el1() -> u64 {
    let x;
    unsafe {
        llvm_asm!("mrs $0, mpidr_el1":"=r"(x));
    }
    x
}

pub fn cpuid() -> usize {
    (mpidr_el1() as usize) & !0xc1000000
}

pub fn isb() {
    unsafe { llvm_asm!("isb":::"memory") }
}

#[inline(always)]
pub fn dsb() {
    unsafe { llvm_asm!("dsb sy":::"memory") }
}

#[inline(always)]
pub fn dmb() {
    unsafe {
        llvm_asm!("dmb sy" ::: "memory": "volatile");
    }
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
    let esr;
    unsafe {
        llvm_asm!("mrs $0, esr_el1":"=r"(esr));
    }
    esr
}

#[inline(always)]
pub fn get_far() -> u64 {
    let far;
    unsafe {
        llvm_asm!("mrs $0, far_el1":"=r"(far));
    }
    far
}

pub fn wfe() {
    unsafe {
        llvm_asm!("wfe");
    }
}

pub fn wfi() {
    unsafe {
        llvm_asm!("wfi");
    }
}

pub fn dc_clean_by_va_PoU(vaddr: usize) {
    unsafe {
        llvm_asm!("dc cvau, $0":: "r"(vaddr));
    }
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
    unsafe { llvm_asm!("ic iallu") }
    isb();
    dsb();
}
