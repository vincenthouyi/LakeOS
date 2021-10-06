use crate::{Error, Result};
use crate::{VirtAddr, PhysAddr};
use crate::common::*;
use crate::permission::Permission;
use crate::arch::{Table, Entry, clean_dcache_by_va, TopLevel};

#[derive(Debug)]
pub struct VSpace<'a, const O: usize> {
    root: &'a mut Table<TopLevel, O>,
}

impl<'a, const O: usize> VSpace<'a, O> {
    pub fn from_root(root: &'a mut Table<TopLevel, O>) -> Self {
        Self { root }
    }

    pub fn into_root(self) -> &'a mut Table<TopLevel, O> {
        self.root
    }

    pub fn root_paddr(&self) -> PhysAddr<O> {
        self.root.paddr()
    }

    pub fn lookup_slot<L: TableLevel>(&self, vaddr: VirtAddr<O>) -> Result<&Entry<L, O>> {
        self.root
            .lookup_slot(vaddr)
    }

    pub fn lookup_slot_mut<L: TableLevel>(&mut self, vaddr: VirtAddr<O>) -> Result<&mut Entry<L, O>> {
        self.root
            .lookup_slot_mut(vaddr)
    }

    pub fn map_entry<L>(&mut self, vaddr: VirtAddr<O>, entry: Entry<L, O>)
        -> Result<()> where L: TableLevel {
        let slot = self.lookup_slot_mut(vaddr)?;
        if slot.is_valid() {
            return Err(Error::SlotOccupied { level: L::LEVEL })
        }
        *slot = entry.into();
        Ok(())
    }

    pub fn unmap_entry<L>(&mut self, vaddr: VirtAddr<O>)
        -> Result<()> where L: TableLevel {
        let slot = self.lookup_slot_mut(vaddr)?;
        if !slot.is_valid() {
            return Err(Error::SlotEmpty);
        }
        *slot = Entry::<L, O>::zero();
        clean_dcache_by_va(slot as *const _ as usize);
        Ok(())
    }

    pub fn map_table<L: TableLevel>(&mut self, vaddr: VirtAddr<O>, table_paddr: PhysAddr<O>) -> Result<()> {
        let entry = Entry::<L, O>::table_entry(table_paddr);
        self.map_entry(vaddr, entry)
    }

    pub fn unmap_table<L: TableLevel>(&mut self, vaddr: VirtAddr<O>) -> Result<()> {
        self.unmap_entry::<L>(vaddr)
    }

    pub fn map_normal_frame<L>(&mut self, vaddr: VirtAddr<O>, frame_paddr: PhysAddr<O>, perm: Permission) -> Result<()>
        where L: PageLevel {
        let entry = Entry::normal_page_entry(frame_paddr, perm);
        self.map_entry::<L>(vaddr, entry)
    }

    pub fn map_device_frame<L>(&mut self, vaddr: VirtAddr<O>, frame_paddr: PhysAddr<O>, perm: Permission) -> Result<()>
        where L: PageLevel {
        let entry = Entry::device_page_entry(frame_paddr, perm);
        self.map_entry::<L>(vaddr, entry)
    }

    pub fn unmap_frame<L: TableLevel>(&mut self, vaddr: VirtAddr<O>) -> Result<()> {
        self.unmap_entry::<L>(vaddr)
    }

    pub fn paddr_of_vaddr(&self, vaddr: VirtAddr<O>) -> Option<PhysAddr<O>> {
        let pgde = self.lookup_slot::<Level4>(vaddr).ok()?;
        if !pgde.is_table_entry() {
            return None
        }

        let pud = pgde.as_table().unwrap();
        let pude = pud.lookup_slot::<Level3>(vaddr).ok()?;
        if !pude.is_valid() {
            return None
        }
        if !pude.is_table_entry() {
            return Some(pude.paddr())
        }

        let pd = pude.as_table().unwrap();
        let pde = pd.lookup_slot::<Level2>(vaddr).ok()?;
        if !pde.is_valid() {
            return None
        }
        if !pde.is_table_entry() {
            return Some(pde.paddr())
        }

        let pt = pde.as_table().unwrap();
        let pte = pt.lookup_slot::<Level1>(vaddr).ok()?;
        if !pte.is_valid() {
            return None
        }
        return Some(pte.paddr())
    }
}
