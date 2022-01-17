use crate::page_table_entry::{Entry, PageTableEntry};
use crate::{Error, PhysAddr, Result, TableLevel, VirtAddr};
use core::fmt::{self, Debug, Formatter};
use core::ops::{Index, IndexMut};
use core::slice;

#[repr(transparent)]
pub struct Table<'a, L: TableLevel, E: PageTableEntry> {
    entries: &'a mut [Entry<L, E>],
}

impl<'a, L: TableLevel, E: PageTableEntry> Index<usize> for Table<'a, L, E> {
    type Output = Entry<L, E>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl<'a, L: TableLevel, E: PageTableEntry> IndexMut<usize> for Table<'a, L, E> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl<'a, L: TableLevel, E: PageTableEntry> Debug for Table<'a, L, E> {
    fn fmt(&self, _f: &mut Formatter) -> fmt::Result {
        Ok(())
    }
}

impl<'a, L: TableLevel, E: PageTableEntry> Table<'a, L, E> {
    pub unsafe fn from_vaddr(ptr: *mut u8) -> Self {
        Self {
            entries: slice::from_raw_parts_mut(ptr as *mut Entry<L, E>, L::TABLE_ENTRIES),
        }
    }

    pub fn paddr<const O: usize>(&self) -> PhysAddr {
        self.vaddr::<O>().into()
    }

    pub fn vaddr<const O: usize>(&self) -> VirtAddr<O> {
        VirtAddr::<O>::from(&self.entries[0])
    }
}

pub trait PageTableExt<E: PageTableEntry, M: TableLevel> {
    fn lookup_slot_mut<'b, const O: usize>(
        &mut self,
        vaddr: VirtAddr<O>,
    ) -> Result<&'b mut Entry<M, E>>;
    fn lookup_slot<'b, const O: usize>(&self, vaddr: VirtAddr<O>) -> Result<&'b Entry<M, E>>;
}

impl<'a, L: TableLevel, M: TableLevel, E: PageTableEntry> PageTableExt<E, M> for Table<'a, L, E> {
    default fn lookup_slot_mut<'b, const O: usize>(
        &mut self,
        vaddr: VirtAddr<O>,
    ) -> Result<&'b mut Entry<M, E>> {
        let idx = vaddr.table_index::<L>();
        let entry = &mut self[idx];
        if M::LEVEL == L::LEVEL {
            return Ok(unsafe { core::mem::transmute(entry) });
        } else {
            panic!()
        }
    }

    default fn lookup_slot<'b, const O: usize>(
        &self,
        vaddr: VirtAddr<O>,
    ) -> Result<&'b Entry<M, E>> {
        let idx = vaddr.table_index::<L>();
        let entry = &self[idx];
        if M::LEVEL == L::LEVEL {
            return Ok(unsafe { core::mem::transmute(entry) });
        } else {
            panic!()
        }
    }
}

impl<'a, L: TableLevel, M: TableLevel, E: PageTableEntry> PageTableExt<E, M> for Table<'a, L, E>
where
    L::NextLevel: TableLevel,
{
    fn lookup_slot_mut<'b, const O: usize>(
        &mut self,
        vaddr: VirtAddr<O>,
    ) -> Result<&'b mut Entry<M, E>> {
        let idx = vaddr.table_index::<L>();
        let entry = &mut self[idx];
        if M::LEVEL == L::LEVEL {
            return Ok(unsafe { core::mem::transmute(entry) });
        } else if M::LEVEL < L::LEVEL {
            let mut next_table = entry
                .as_table_mut::<O>()
                .ok_or(Error::TableMiss { level: L::LEVEL })?;
            return next_table.lookup_slot_mut(vaddr);
        } else {
            panic!()
        }
    }

    fn lookup_slot<'b, const O: usize>(&self, vaddr: VirtAddr<O>) -> Result<&'b Entry<M, E>> {
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
