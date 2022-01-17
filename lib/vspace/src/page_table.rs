use crate::page_table_entry::{Entry, PageTableEntry};
use crate::{Error, PhysAddr, Result, TableLevel, VirtAddr};
use core::fmt::{self, Debug, Formatter};
use core::ops::{Index, IndexMut};

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Table<L: TableLevel, E: PageTableEntry> {
    entries: [Entry<L, E>; 512],
}

impl<L: TableLevel, E: PageTableEntry> Index<usize> for Table<L, E> {
    type Output = Entry<L, E>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl<L: TableLevel, E: PageTableEntry> IndexMut<usize> for Table<L, E> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl<L: TableLevel, E: PageTableEntry> Debug for Table<L, E> {
    fn fmt(&self, _f: &mut Formatter) -> fmt::Result {
        Ok(())
    }
}

impl<L: TableLevel, E: PageTableEntry> Table<L, E> {
    pub unsafe fn from_vaddr<'a>(ptr: *mut u8) -> &'a mut Self {
        &mut *(ptr as *mut Self)
    }

    pub fn paddr<const O: usize>(&self) -> PhysAddr {
        VirtAddr::<O>::from(self).into()
    }

    pub fn vaddr<const O: usize>(&self) -> VirtAddr<O> {
        VirtAddr::<O>::from(self)
    }
}

pub trait PageTableExt<E: PageTableEntry, M: TableLevel> {
    fn lookup_slot_mut<const O: usize>(&mut self, vaddr: VirtAddr<O>) -> Result<&mut Entry<M, E>>;
    fn lookup_slot<const O: usize>(&self, vaddr: VirtAddr<O>) -> Result<&Entry<M, E>>;
}

impl<L: TableLevel, M: TableLevel, E: PageTableEntry> PageTableExt<E, M> for Table<L, E> {
    default fn lookup_slot_mut<const O: usize>(
        &mut self,
        vaddr: VirtAddr<O>,
    ) -> Result<&mut Entry<M, E>> {
        let idx = vaddr.table_index::<L>();
        let entry = &mut self[idx];
        if M::LEVEL == L::LEVEL {
            return Ok(unsafe { core::mem::transmute(entry) });
        } else {
            panic!()
        }
    }

    default fn lookup_slot<const O: usize>(&self, vaddr: VirtAddr<O>) -> Result<&Entry<M, E>> {
        let idx = vaddr.table_index::<L>();
        let entry = &self[idx];
        if M::LEVEL == L::LEVEL {
            return Ok(unsafe { core::mem::transmute(entry) });
        } else {
            panic!()
        }
    }
}

impl<L: TableLevel, M: TableLevel, E: PageTableEntry> PageTableExt<E, M> for Table<L, E>
where
    L::NextLevel: TableLevel,
{
    fn lookup_slot_mut<const O: usize>(&mut self, vaddr: VirtAddr<O>) -> Result<&mut Entry<M, E>> {
        let idx = vaddr.table_index::<L>();
        let entry = &mut self[idx];
        if M::LEVEL == L::LEVEL {
            return Ok(unsafe { core::mem::transmute(entry) });
        } else if M::LEVEL < L::LEVEL {
            let next_table = entry
                .as_table_mut::<O>()
                .ok_or(Error::TableMiss { level: L::LEVEL })?;
            return next_table.lookup_slot_mut(vaddr);
        } else {
            panic!()
        }
    }

    fn lookup_slot<const O: usize>(&self, vaddr: VirtAddr<O>) -> Result<&Entry<M, E>> {
        let idx = vaddr.table_index::<L>();
        let entry = &self[idx];
        if M::LEVEL == L::LEVEL {
            return Ok(unsafe { core::mem::transmute(entry) });
        } else if M::LEVEL < L::LEVEL {
            let next_table = entry
                .as_table::<O>()
                .ok_or(Error::TableMiss { level: L::LEVEL })?;
            return next_table.lookup_slot(vaddr);
        } else {
            panic!()
        }
    }
}
