use core::borrow::BorrowMut;
use crate::{Error, Result};
use crate::{VirtAddr, PhysAddr};
use crate::common::*;
use crate::permission::Permission;
use crate::arch::{Table, Entry, clean_dcache_by_va, TopLevel};

#[derive(Debug)]
pub struct VSpace<R> {
    root: R,
}

impl<R: BorrowMut<Table<TopLevel>>> VSpace<R> {
    pub fn from_root(root: R) -> Self {
        Self { root }
    }

    pub fn into_root(self) -> R {
        self.root
    }

    pub fn lookup_slot<L: TableLevel, V: Into<VirtAddr>>(&self, vaddr: V) -> Result<&Entry<L>> {
        self.root
            .borrow()
            .lookup_slot(vaddr)
    }

    pub fn lookup_slot_mut<L: TableLevel, V: Into<VirtAddr>>(&mut self, vaddr: V) -> Result<&mut Entry<L>> {
        self.root
            .borrow_mut()
            .lookup_slot_mut(vaddr)
    }

    pub fn map_entry<L, V, E>(&mut self, vaddr: V, entry: E)
        -> Result<()> where L: TableLevel, V: Into<VirtAddr>, E: Into<Entry<L>> {
        let slot = self.lookup_slot_mut(vaddr)?;
        if slot.is_valid() {
            return Err(Error::SlotOccupied { level: L::LEVEL })
        }
        *slot = entry.into();
        Ok(())
    }

    pub fn unmap_entry<L, V>(&mut self, vaddr: V)
        -> Result<()> where L: TableLevel, V: Into<VirtAddr> {
        let slot = self.lookup_slot_mut(vaddr.into())?;
        if !slot.is_valid() {
            return Err(Error::SlotEmpty);
        }
        *slot = Entry::<L>::zero();
        clean_dcache_by_va(slot as *const _ as usize);
        Ok(())
    }

    pub fn map_table<L: TableLevel>(&mut self, vaddr: VirtAddr, table_paddr: PhysAddr) -> Result<()> {
        let entry = Entry::<L>::table_entry(table_paddr);
        self.map_entry(vaddr, entry)
    }

    pub fn unmap_table<L: TableLevel, V: Into<VirtAddr>>(&mut self, vaddr: V) -> Result<()> {
        self.unmap_entry::<L, V>(vaddr)
    }

    pub fn map_normal_frame<L, V>(&mut self, vaddr: V, frame_paddr: PhysAddr, perm: Permission) -> Result<()>
        where L: PageLevel, V: Into<VirtAddr> {
        let entry = Entry::normal_page_entry(frame_paddr, perm);
        self.map_entry::<L, V, Entry<L>>(vaddr, entry)
    }

    pub fn unmap_frame<L: TableLevel, V: Into<VirtAddr>>(&mut self, vaddr: V) -> Result<()> {
        self.unmap_entry::<L, V>(vaddr)
    }
}
