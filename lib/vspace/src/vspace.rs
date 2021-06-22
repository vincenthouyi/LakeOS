use crate::{Error, Result};
use crate::{VirtAddr, PhysAddr};
use crate::common::*;
use crate::permission::Permission;
use crate::arch::{Table, Entry, clean_dcache_by_va, TopLevel};

#[derive(Debug)]
pub struct VSpace<'a> {
    root: &'a mut Table<TopLevel>,
}

impl<'a> VSpace<'a> {
    pub fn from_root(root: &'a mut Table<TopLevel>) -> Self {
        Self { root }
    }

    pub fn into_root(self) -> &'a mut Table<TopLevel> {
        self.root
    }

    pub fn root_paddr(&self) -> PhysAddr {
        self.root.paddr()
    }

    pub fn lookup_slot<L: TableLevel, V: Into<VirtAddr>>(&self, vaddr: V) -> Result<&Entry<L>> {
        self.root
            .lookup_slot(vaddr)
    }

    pub fn lookup_slot_mut<L: TableLevel, V: Into<VirtAddr>>(&mut self, vaddr: V) -> Result<&mut Entry<L>> {
        self.root
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

    pub fn map_device_frame<L, V>(&mut self, vaddr: V, frame_paddr: PhysAddr, perm: Permission) -> Result<()>
        where L: PageLevel, V: Into<VirtAddr> {
        let entry = Entry::device_page_entry(frame_paddr, perm);
        self.map_entry::<L, V, Entry<L>>(vaddr, entry)
    }

    pub fn unmap_frame<L: TableLevel, V: Into<VirtAddr>>(&mut self, vaddr: V) -> Result<()> {
        self.unmap_entry::<L, V>(vaddr)
    }
}
