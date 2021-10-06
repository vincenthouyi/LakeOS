use core::ops::{Index, IndexMut};
use core::default::Default;
use core::marker::PhantomData;
use core::fmt::{self, Debug, Formatter};

use crate::{Error, Result};
use crate::{VirtAddr, PhysAddr};
use crate::common::*;
use crate::permission::Permission;

use super::mmu::{MemoryAttr, AccessPermission, Shareability};

const PADDR_MASK: usize = MASK!(48) & (!MASK!(12));
const VALID_OFFSET: usize = 0;
const TABLE_OFFSET: usize = 1;
const UXN_OFFSET: usize = 54;
const N_G_OFFSET: usize = 11;
const AF_OFFSET: usize = 10;
const ATTR_INDEX_OFFSET: usize = 2;

#[derive(Default, Debug, Copy, Clone)]
pub struct Entry<L: TableLevel, const O:usize> {
    inner: usize,
    level: PhantomData<L>,
}

impl<L: TableLevel, const O:usize> Entry<L, O> {
    pub const fn zero() -> Self {
        Self::new(0)
    }

    pub const fn new(entry: usize) -> Self {
        Self{
            inner: entry,
            level: PhantomData,
        }
    }

    #[inline(always)]
    pub const fn table_entry(paddr: PhysAddr<O>) -> Self {
        Self::new((paddr.0 & PADDR_MASK) | 1 << TABLE_OFFSET | 1 << VALID_OFFSET)
    }

    pub const fn is_valid(&self) -> bool {
        self.inner & 0x1 == 0x1
    }

    pub const fn paddr(&self) -> PhysAddr<O> {
        PhysAddr::<O>(self.inner & PADDR_MASK)
    }

    pub const fn vaddr(&self) -> VirtAddr<O> {
        crate::addr::phys_to_virt(self.paddr())
    }

    pub const fn is_table_entry(&self) -> bool {
        if !self.is_valid() {
            return false;
        }

        return L::LEVEL != 1 && (self.inner & 1 << TABLE_OFFSET == 1 << TABLE_OFFSET);
    }
}

impl<L: PageLevel, const O: usize> Entry<L, O> {
    #[inline(always)]
    pub const fn page_entry(
        paddr: PhysAddr<O>,
        uxn: bool,
        global: bool,
        af: bool,
        share: Shareability,
        ap: AccessPermission,
        attr: MemoryAttr,
    ) -> Self {
        if L::LEVEL == 1 {
            Self::new(
                (paddr.0 & PADDR_MASK)
                | (uxn as usize) << UXN_OFFSET         // universal execute never
                | ((!global) as usize) << N_G_OFFSET   // nG bit
                | (af as usize) << AF_OFFSET           // access flag
                | share as usize                       // Shareability
                | ap as usize                          // access permission
                | (attr as usize) << ATTR_INDEX_OFFSET // mair index
                | 1 << 1
                | 1 << VALID_OFFSET,
            )
        } else {
            Self::new(
                (paddr.0 & PADDR_MASK)
                | (uxn as usize) << UXN_OFFSET         // universal execute never
                | ((!global) as usize) << N_G_OFFSET   // nG bit
                | (af as usize) << AF_OFFSET           // access flag
                | share as usize                       // Shareability
                | ap as usize                          // access permission
                | (attr as usize) << ATTR_INDEX_OFFSET // mair index
                | 1 << VALID_OFFSET,
            )
        }
    }

    pub fn normal_page_entry(
        paddr: PhysAddr<O>,
        perm: Permission,
    ) -> Self {
        let is_executable = perm.is_executable();
        Self::page_entry(
            paddr,
            !is_executable,
            false,
            true,
            Shareability::InnerSharable,
            perm.into(),
            MemoryAttr::Normal
        )
    }

    pub fn device_page_entry(
        paddr: PhysAddr<O>,
        perm: Permission,
    ) -> Self {
        Self::page_entry(
            paddr,
            true,
            false,
            true,
            Shareability::NonSharable,
            perm.into(),
            MemoryAttr::DevicenGnRnE,
        )
    }
}

impl<L: TableLevel, const O: usize> Entry<L, O> {
    pub fn as_table(&self) -> Option<&Table<L::NextLevel, O>> {
        self.is_table_entry()
            .then_some(
                unsafe { &*(VirtAddr::<O>::from(self.paddr()).0 as *const _) }
            )
    }

    pub fn as_table_mut(&mut self) -> Option<&mut Table<L::NextLevel, O>> {
        self.is_table_entry()
            .then_some(
                unsafe { &mut *(VirtAddr::<O>::from(self.paddr()).0 as *mut _) }
            )
    }
}

pub type TopLevel = Level4;

#[derive(Copy, Clone)]
#[repr(align(4096))]
pub struct Table<L: TableLevel, const O: usize> {
    entries: [Entry<L, O>; 512],
}

impl<L: TableLevel, const O: usize> Default for Table<L, O> where Entry<L, O>: Copy {
    fn default() -> Self {
        Self::zero()
    }
}

impl<L: TableLevel, const O: usize> Index<usize> for Table<L, O> {
    type Output = Entry<L, O>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl<L: TableLevel, const O: usize> IndexMut<usize> for Table<L, O> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl<L: TableLevel, const O: usize> Debug for Table<L, O> {
    fn fmt(&self, _f: &mut Formatter) -> fmt::Result {
        Ok(())
    }
}

impl<L: TableLevel, const O: usize> Table<L, O> where Entry<L, O>: Copy {
    pub const fn zero() -> Self {
        Self {
            entries: [Entry::<L, O>::zero(); 512],
        }
    }
}

impl <L: TableLevel, const O: usize> Table<L, O> {
    pub unsafe fn from_vaddr<'a>(ptr: *mut u8) -> &'a mut Self {
        &mut *(ptr as *mut Self)
    }

    pub fn lookup_slot_mut<M: TableLevel>(&mut self, vaddr: VirtAddr<O>) -> Result<&mut Entry<M, O>> {
        let idx = vaddr.table_index::<L>();
        let entry = &mut self[idx];
        if M::LEVEL == L::LEVEL {
            return Ok(unsafe { core::mem::transmute(entry) });
        } else if M::LEVEL < L::LEVEL {
            let next_table = entry.as_table_mut()
                                  .ok_or(Error::TableMiss { level: L::LEVEL } )?;
            return next_table.lookup_slot_mut(vaddr);
        } else {
            panic!()
        }
    }

    pub fn lookup_slot<M: TableLevel>(&self, vaddr: VirtAddr<O>) -> Result<&Entry<M, O>> {
        let idx = vaddr.table_index::<L>();
        let entry = &self[idx];
        if M::LEVEL == L::LEVEL {
            return Ok(unsafe { core::mem::transmute(entry) });
        } else if M::LEVEL < L::LEVEL {
            let next_table = entry.as_table()
                                  .ok_or(Error::TableMiss { level: L::LEVEL } )?;
            return next_table.lookup_slot(vaddr);
        } else {
            panic!()
        }
    }

    pub fn paddr(&self) -> PhysAddr<O> {
        VirtAddr::<O>::from(self).into()
    }

    pub fn vaddr(&self) -> VirtAddr<O> {
        VirtAddr::<O>::from(self)
    }
}
