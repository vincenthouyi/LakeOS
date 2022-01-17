mod page_table;
pub use page_table::*;

pub mod mmu;

pub mod asm;
mod page;

pub fn clean_dcache_by_va(vaddr: usize) {
    asm::dc_clean_by_va_pou(vaddr)
}

use crate::{Entry as _Entry, Level, PageLevel, Table as _Table, TableLevel, VSpace as _VSpace};

#[derive(Copy, Clone, Debug)]
pub enum Level4 {}
#[derive(Copy, Clone, Debug)]
pub enum Level3 {}
#[derive(Copy, Clone, Debug)]
pub enum Level2 {}
#[derive(Copy, Clone, Debug)]
pub enum Level1 {}
#[derive(Copy, Clone, Debug)]
pub enum Level0 {}

impl Level for Level4 {
    const LEVEL: usize = 4;
}
impl Level for Level3 {
    const LEVEL: usize = 3;
}
impl Level for Level2 {
    const LEVEL: usize = 2;
}
impl Level for Level1 {
    const LEVEL: usize = 1;
}
impl Level for Level0 {
    const LEVEL: usize = 1;
}

impl TableLevel for Level4 {
    type NextLevel = Level3;
    const TABLE_ENTRIES: usize = 512;
}
impl TableLevel for Level3 {
    type NextLevel = Level2;
    const TABLE_ENTRIES: usize = 512;
}
impl TableLevel for Level2 {
    type NextLevel = Level1;
    const TABLE_ENTRIES: usize = 512;
}
impl TableLevel for Level1 {
    type NextLevel = Level0;
    const TABLE_ENTRIES: usize = 512;
}

impl PageLevel for Level2 {
    const FRAME_BIT_SIZE: usize = 30;
}

impl PageLevel for Level1 {
    const FRAME_BIT_SIZE: usize = 21;
}

impl PageLevel for Level0 {
    const FRAME_BIT_SIZE: usize = 12;
}

pub type ArchEntry = page_table::Aarch64PageTableEntry;
pub type Table<'a, L> = _Table<'a, L, ArchEntry>;
pub type PageTable<'a> = Table<'a, Level1>;
pub type PageDirectory<'a> = Table<'a, Level2>;
pub type PageUpperDirectory<'a> = Table<'a, Level3>;
pub type PageGlobalDirectory<'a> = Table<'a, Level4>;
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
