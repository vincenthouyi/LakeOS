use crate::page_table_entry::Entry;
use crate::{Error, PhysAddr, Result, TableLevel, VirtAddr};
use core::fmt::{self, Debug, Formatter};
use core::ops::{Index, IndexMut};
use core::slice;

#[repr(transparent)]
pub struct Table<'a, L: TableLevel> {
    entries: &'a mut [Entry<L>],
}

impl<'a, L: TableLevel> Index<usize> for Table<'a, L> {
    type Output = Entry<L>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl<'a, L: TableLevel> IndexMut<usize> for Table<'a, L> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl<'a, L: TableLevel> Debug for Table<'a, L> {
    fn fmt(&self, _f: &mut Formatter) -> fmt::Result {
        Ok(())
    }
}

impl<'a, L: TableLevel> Table<'a, L> {
    pub unsafe fn from_vaddr(ptr: *mut u8) -> Self {
        Self {
            entries: slice::from_raw_parts_mut(ptr as *mut Entry<L>, L::TABLE_ENTRIES),
        }
    }

    pub fn paddr<const O: usize>(&self) -> PhysAddr {
        self.vaddr::<O>().into()
    }

    pub fn vaddr<const O: usize>(&self) -> VirtAddr<O> {
        VirtAddr::<O>::from(&self.entries[0])
    }
}

pub trait PageTableExt<M: TableLevel> {
    fn lookup_slot_mut<'b, const O: usize>(
        &mut self,
        vaddr: VirtAddr<O>,
    ) -> Result<&'b mut Entry<M>>;
    fn lookup_slot<'b, const O: usize>(&self, vaddr: VirtAddr<O>) -> Result<&'b Entry<M>>;
}

impl<'a, L: TableLevel, M: TableLevel> PageTableExt<M> for Table<'a, L> {
    default fn lookup_slot_mut<'b, const O: usize>(
        &mut self,
        vaddr: VirtAddr<O>,
    ) -> Result<&'b mut Entry<M>> {
        let idx = vaddr.table_index::<L>();
        let entry = &mut self[idx];
        Ok(entry.transmute_mut())
    }

    default fn lookup_slot<'b, const O: usize>(&self, vaddr: VirtAddr<O>) -> Result<&'b Entry<M>> {
        let idx = vaddr.table_index::<L>();
        let entry = &self[idx];
        Ok(entry.transmute())
    }
}

impl<'a, L: TableLevel, M: TableLevel> PageTableExt<M> for Table<'a, L>
where
    L::NextLevel: TableLevel,
{
    fn lookup_slot_mut<'b, const O: usize>(
        &mut self,
        vaddr: VirtAddr<O>,
    ) -> Result<&'b mut Entry<M>> {
        let idx = vaddr.table_index::<L>();
        let entry = &mut self[idx];
        if M::LEVEL == L::LEVEL {
            Ok(entry.transmute_mut())
        } else if M::LEVEL < L::LEVEL {
            entry
                .as_table_mut::<O>()
                .ok_or(Error::TableMiss { level: L::LEVEL })
                .and_then(|mut e| e.lookup_slot_mut(vaddr))
        } else {
            panic!()
        }
    }

    fn lookup_slot<'b, const O: usize>(&self, vaddr: VirtAddr<O>) -> Result<&'b Entry<M>> {
        let idx = vaddr.table_index::<L>();
        let entry = &self[idx];
        if M::LEVEL == L::LEVEL {
            Ok(entry.transmute())
        } else if M::LEVEL < L::LEVEL {
            entry
                .as_table::<O>()
                .ok_or(Error::TableMiss { level: L::LEVEL })
                .and_then(|e| e.lookup_slot(vaddr))
        } else {
            panic!()
        }
    }
}
