use crate::addr::{PhysAddr, VirtAddr};
use crate::page_table::Table;
use crate::TableLevel;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::Deref;
pub trait PageTableEntry: Copy + Clone + Debug {
    fn invalid_entry<L: TableLevel>() -> Self;
    fn is_valid<L: TableLevel>(&self) -> bool;
    fn paddr<L: TableLevel>(&self) -> PhysAddr;
    fn is_table_entry<L: TableLevel>(&self) -> bool;
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Entry<L, E> {
    inner: E,
    level: PhantomData<L>,
}

impl<L: TableLevel, E: PageTableEntry> Entry<L, E> {
    pub fn new(entry: E) -> Self {
        Self {
            inner: entry,
            level: PhantomData,
        }
    }

    pub fn invalid_entry() -> Self {
        Self::new(E::invalid_entry::<L>())
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

    pub fn raw(&self) -> E {
        self.inner
    }
}

impl<L: TableLevel, E: PageTableEntry> Entry<L, E>
where
    L::NextLevel: TableLevel,
{
    pub fn as_table<const O: usize>(&self) -> Option<&Table<L::NextLevel, E>> {
        self.is_table_entry()
            .then_some(unsafe { &*(self.vaddr::<O>().0 as *const _) })
    }

    pub fn as_table_mut<const O: usize>(&mut self) -> Option<&mut Table<L::NextLevel, E>> {
        self.is_table_entry()
            .then_some(unsafe { &mut *(self.vaddr::<O>().0 as *mut _) })
    }
}

impl<L: TableLevel, E: PageTableEntry> Deref for Entry<L, E> {
    type Target = E;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<L: TableLevel, E: PageTableEntry> From<E> for Entry<L, E> {
    fn from(inner: E) -> Self {
        Self::new(inner)
    }
}
