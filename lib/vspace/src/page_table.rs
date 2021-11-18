use core::fmt::{self, Debug, Formatter};
use core::marker::PhantomData;
use core::ops::{Index, IndexMut, Deref};
use crate::{PhysAddr, VirtAddr, TableLevel, Error, Result};

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

impl<L: TableLevel, E: PageTableEntry> Entry<L, E> where L::NextLevel: TableLevel {
    pub fn as_table<const O: usize>(&self) -> Option<&Table<L::NextLevel, E>> {
        self.is_table_entry()
            .then_some(
                unsafe { &*(self.vaddr::<O>().0 as *const _) }
            )
    }

    pub fn as_table_mut<const O: usize>(&mut self) -> Option<&mut Table<L::NextLevel, E>> {
        self.is_table_entry()
            .then_some(
                unsafe { &mut *(self.vaddr::<O>().0 as *mut _) }
            )
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

impl <L: TableLevel, E: PageTableEntry> Table<L, E> {
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

impl <L: TableLevel, M: TableLevel, E: PageTableEntry> PageTableExt<E, M> for Table<L, E> {
    default fn lookup_slot_mut<const O: usize>(&mut self, vaddr: VirtAddr<O>) -> Result<&mut Entry<M, E>> {
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

impl <L: TableLevel, M: TableLevel, E: PageTableEntry> PageTableExt<E, M> for Table<L, E> where L::NextLevel: TableLevel {
    fn lookup_slot_mut<const O: usize>(&mut self, vaddr: VirtAddr<O>) -> Result<&mut Entry<M, E>> {
        let idx = vaddr.table_index::<L>();
        let entry = &mut self[idx];
        if M::LEVEL == L::LEVEL {
            return Ok(unsafe { core::mem::transmute(entry) });
        } else if M::LEVEL < L::LEVEL {
            let next_table = entry.as_table_mut::<O>()
                                  .ok_or(Error::TableMiss { level: L::LEVEL } )?;
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
            let next_table = entry.as_table::<O>()
                                  .ok_or(Error::TableMiss { level: L::LEVEL } )?;
            return next_table.lookup_slot(vaddr);
        } else {
            panic!()
        }
    }
}