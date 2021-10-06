use crate::{Table};
use crate::arch::TopLevel;

pub unsafe fn flush_all_tlb() {
    crate::arch::asm::flush_tlb_allel1_is()
}

pub unsafe fn init_mmu() {
    crate::arch::mmu::init_mmu()
}

pub unsafe fn install_user_vspace<const O: usize>(root_table: &Table<TopLevel, O>, asid: usize) {
    crate::arch::mmu::install_user_vspace(asid, root_table.paddr().0)
}

pub unsafe fn install_kernel_vspace<const O: usize>(root_table: &Table<TopLevel, O>, _asid: usize) {
    crate::arch::mmu::install_kernel_vspace(root_table.paddr())
}

pub unsafe fn invalidate_tlb_by_asid(asid: usize) {
    crate::arch::mmu::invalidate_local_tlb_asid(asid)
}