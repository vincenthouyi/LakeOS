use core::arch::asm;

pub fn dc_clean_by_va_pou(vaddr: usize) {
    unsafe {
        asm!("dc cvau, {vaddr}", vaddr = in(reg) vaddr, options(nomem));
    }
    dsb();
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

// #[inline(always)]
// #[allow(dead_code)]
// pub fn get_current_user_vspace() -> usize {
//    let ret;
//    unsafe {
//        llvm_asm!("mrs $0, ttbr0_el1": "=r"(ret));
//    }
//    ret
// }

#[inline(always)]
pub unsafe fn flush_tlb_allel1_is() {
    dsb();
    asm!(
        "
        dsb ishst
        tlbi vmalle1is
        dsb ish
    ",
        options(nomem),
    );
    isb();
}

// pub fn invalidateLocalTLB_ASID(asid: usize) {
//     dsb();
//     unsafe { llvm_asm!("tlbi aside1, $0"::"r"(asid)) }
//     dsb();
//     isb();
// }

#[inline(always)]
pub unsafe fn set_mair(mair: usize) {
    asm!("msr mair_el1, {mair}",
         mair = in(reg) mair, options(nomem));
    isb();
}
