
mod page_table;
pub use page_table::*;

pub mod mmu;

pub mod asm;
mod page;

pub fn clean_dcache_by_va(vaddr: usize) {
    asm::dc_clean_by_va_pou(vaddr)
}

use crate::{VSpace as _VSpace, Level1, Level2, Level3, Level4, Table as _Table, Entry as _Entry};

pub type ArchEntry = page_table::Aarch64PageTableEntry;
pub type Table<L> = _Table<L, ArchEntry>;
pub type PageTable = Table<Level1>;
pub type PageDirectory = Table<Level2>;
pub type PageUpperDirectory = Table<Level3>;
pub type PageGlobalDirectory = Table<Level4>;
pub type Entry<L> = _Entry<L, ArchEntry>;
pub type PTE = Entry<Level1>;
pub type PDE = Entry<Level2>;
pub type PUDE = Entry<Level3>;
pub type PGDE = Entry<Level4>;
pub type TopLevel = Level4;
pub type VSpace<'a, const O: usize> = _VSpace<'a, TopLevel, ArchEntry, O>;

impl<'a, const O: usize> VSpace<'a, O> {
    pub unsafe fn install_user_vspace(&self, asid: usize) {
        mmu::install_user_vspace(asid, self.root_paddr().0)
    }

    pub unsafe fn invalidate_tlb_by_asid(&self, asid: usize) {
        mmu::invalidate_local_tlb_asid(asid)
    }
}