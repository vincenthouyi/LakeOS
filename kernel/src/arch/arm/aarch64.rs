use core::arch::asm;

pub fn mpidr_el1() -> u64 {
    let x;
    unsafe {
        asm!("mrs {x}, mpidr_el1", x = out(reg) x, options(nomem));
    }
    x
}

pub fn cpuid() -> usize {
    (mpidr_el1() as usize) & !0xc1000000
}

pub fn isb() {
    unsafe { asm!("isb", options(nomem)) }
}

#[inline(always)]
pub fn dsb() {
    unsafe { asm!("dsb sy", options(nomem)) }
}

#[inline(always)]
pub fn dmb() {
    unsafe {
        asm!("dmb sy", options(nomem));
    }
}

#[inline(always)]
pub fn get_elr() -> usize {
    let elr;
    unsafe {
        asm!("mrs {elr}, elr_el1", elr = out(reg) elr, options(nomem));
    }
    elr
}

#[inline(always)]
pub fn get_esr() -> u32 {
    let esr;
    unsafe {
        asm!("mrs {esr:x}, esr_el1", esr = out(reg) esr, options(nomem));
    }
    esr
}

#[inline(always)]
pub fn get_far() -> u64 {
    let far;
    unsafe {
        asm!("mrs {far}, far_el1", far = out(reg) far, options(nomem));
    }
    far
}

pub fn wfe() {
    unsafe {
        asm!("wfe", options(nomem));
    }
}

pub fn wfi() {
    unsafe {
        asm!("wfi", options(nomem));
    }
}

pub fn dc_clean_by_va_PoU(vaddr: usize) {
    unsafe {
        asm!("dc cvau, {vaddr}", vaddr = in(reg) vaddr, options(nomem));
    }
    dsb();
}

pub fn clid() -> usize {
    let clid: usize;
    unsafe {
        asm!("mrs {clid}, clidr_el1", clid = out(reg) clid, options(nomem));
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
        asm!("
            mrs {cssr_old}, csselr_el1
            msr csselr_el1, {selector} 
            ", cssr_old = out(reg) cssr_old, selector = in(reg) selector as usize);
        asm!("
            mrs {size}, ccsidr_el1
            msr csselr_el1, {cssr_old}
        ", size = out(reg) size, cssr_old = in(reg) cssr_old, options(nomem))
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
                    unsafe { asm!("dc cisw, {wsl}", wsl = in(reg) wsl, options(nomem)) }
                }
            }
        }
    }
}

pub fn clean_l1_cache() {
    dsb();
    clean_dcache_poc();
    dsb();
    unsafe { asm!("ic iallu", options(nomem)) }
    isb();
    dsb();
}
