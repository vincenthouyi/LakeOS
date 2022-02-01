use crate::arch::clean_dcache_by_va;
use crate::arch::{Level1, Level2, Level3, Level4};
use crate::common::*;
use crate::page_table::{PageTableExt, Table};
use crate::page_table_entry::Entry;
use crate::{Error, Result};
use crate::{PhysAddr, VirtAddr};

#[derive(Debug)]
pub struct VSpace<'a, T: TopLevel, const O: usize> {
    root: Table<'a, T>,
}

impl<'a, T: TopLevel, const O: usize> VSpace<'a, T, O> {
    pub unsafe fn from_vaddr(vaddr: *mut u8) -> Self {
        let root = Table::<T>::from_vaddr(vaddr);
        Self { root }
    }

    pub fn from_root(root: Table<'a, T>) -> Self {
        Self { root }
    }

    pub fn into_root(self) -> Table<'a, T> {
        self.root
    }

    pub fn root_paddr(&self) -> PhysAddr {
        self.root.paddr::<O>()
    }

    pub fn lookup_slot<L: TableLevel>(&self, vaddr: VirtAddr<O>) -> Result<&Entry<L>> {
        self.root.lookup_slot(vaddr)
    }

    pub fn lookup_slot_mut<L: TableLevel>(&mut self, vaddr: VirtAddr<O>) -> Result<&mut Entry<L>> {
        self.root.lookup_slot_mut(vaddr)
    }

    pub fn map_entry<L>(&mut self, vaddr: VirtAddr<O>, entry: L::EntryType) -> Result<()>
    where
        L: TableLevel,
    {
        let slot = self.lookup_slot_mut::<L>(vaddr)?;
        if slot.is_valid() {
            return Err(Error::SlotOccupied { level: L::LEVEL });
        }
        *slot = Entry::new(entry);
        Ok(())
    }

    pub fn unmap_entry<L>(&mut self, vaddr: VirtAddr<O>) -> Result<()>
    where
        L: TableLevel,
    {
        let slot = self.lookup_slot_mut::<L>(vaddr)?;
        if !slot.is_valid() {
            return Err(Error::SlotEmpty);
        }
        *slot = Entry::<L>::invalid_entry();
        clean_dcache_by_va(slot as *const _ as usize);
        Ok(())
    }

    pub fn paddr_of_vaddr(&self, vaddr: VirtAddr<O>) -> Option<PhysAddr> {
        let pgde = self.lookup_slot::<Level4>(vaddr).ok()?;
        if !pgde.is_table_entry() {
            return None;
        }

        let pud = pgde.as_table::<O>().unwrap();
        let pude: &Entry<Level3> = pud.lookup_slot::<O>(vaddr).ok()?;
        if !pude.is_valid() {
            return None;
        }
        if !pude.is_table_entry() {
            return Some(pude.paddr());
        }

        let pd = pude.as_table::<O>().unwrap();
        let pde: &Entry<Level2> = pd.lookup_slot::<O>(vaddr).ok()?;
        if !pde.is_valid() {
            return None;
        }
        if !pde.is_table_entry() {
            return Some(pde.paddr());
        }

        let pt = pde.as_table::<O>().unwrap();
        let pte: &Entry<Level1> = pt.lookup_slot::<O>(vaddr).ok()?;
        if !pte.is_valid() {
            return None;
        }
        return Some(pte.paddr());
    }
}
