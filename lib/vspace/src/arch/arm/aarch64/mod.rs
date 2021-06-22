mod page_table;
pub use page_table::*;

pub mod mmu;

pub mod asm;
mod page;

pub fn clean_dcache_by_va(vaddr: usize) {
    asm::dc_clean_by_va_pou(vaddr)
}
