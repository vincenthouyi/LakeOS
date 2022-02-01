use crate::addr::{PhysAddr, VirtAddr};
use crate::page_table::Table;
use crate::TableLevel;
use core::fmt::Debug;
use core::ops::Deref;
pub trait PageTableEntry: Copy + Clone + Debug {
    fn invalid_entry<L: TableLevel>() -> Self;
    fn is_valid<L: TableLevel>(&self) -> bool;
    fn paddr<L: TableLevel>(&self) -> PhysAddr;
    fn is_table_entry<L: TableLevel>(&self) -> bool;
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Entry<L: TableLevel> {
    inner: L::EntryType,
}

impl<L: TableLevel> Entry<L> {
    pub fn new(entry: L::EntryType) -> Self {
        Self { inner: entry }
    }

    pub fn invalid_entry() -> Self {
        Self::new(L::EntryType::invalid_entry::<L>())
    }

    pub fn is_valid(&self) -> bool {
        self.inner.is_valid::<L>()
    }

    pub fn paddr(&self) -> PhysAddr {
        self.inner.paddr::<L>()
    }

    pub fn vaddr<const O: usize>(&self) -> VirtAddr<O> {
        crate::addr::phys_to_virt(self.paddr())
    }

    pub fn is_table_entry(&self) -> bool {
        if !self.is_valid() {
            return false;
        }

        return self.inner.is_table_entry::<L>();
    }

    pub fn raw(&self) -> L::EntryType {
        self.inner
    }
}

impl<L: TableLevel> Entry<L>
where
    L::NextLevel: TableLevel,
{
    pub fn as_table<const O: usize>(&self) -> Option<Table<L::NextLevel>> {
        self.is_table_entry()
            .then_some(unsafe { Table::<L::NextLevel>::from_vaddr(self.vaddr::<O>().0 as *mut u8) })
    }

    pub fn as_table_mut<const O: usize>(&mut self) -> Option<Table<L::NextLevel>> {
        self.is_table_entry()
            .then_some(unsafe { Table::<L::NextLevel>::from_vaddr(self.vaddr::<O>().0 as *mut u8) })
    }
}

impl<L: TableLevel> Deref for Entry<L> {
    type Target = L::EntryType;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
