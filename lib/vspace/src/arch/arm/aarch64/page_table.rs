use crate::common::*;
use crate::page_table_entry::PageTableEntry;
use crate::permission::Permission;
use crate::{PhysAddr, VirtAddr};

use super::mmu::{AccessPermission, MemoryAttr, Shareability};

const PADDR_MASK: usize = MASK!(48) & (!MASK!(12));
const VALID_OFFSET: usize = 0;
const TABLE_OFFSET: usize = 1;
const UXN_OFFSET: usize = 54;
const N_G_OFFSET: usize = 11;
const AF_OFFSET: usize = 10;
const ATTR_INDEX_OFFSET: usize = 2;

#[derive(Debug, Clone, Copy)]
pub struct Aarch64PageTableEntry(u64);

impl Aarch64PageTableEntry {
    #[inline(always)]
    pub const fn table_entry(paddr: PhysAddr) -> Self {
        Self(((paddr.0 & PADDR_MASK) | 1 << TABLE_OFFSET | 1 << VALID_OFFSET) as u64)
    }

    pub const fn is_valid(&self) -> bool {
        self.0 & 0x1 == 0x1
    }

    pub const fn paddr(&self) -> PhysAddr {
        PhysAddr(self.0 as usize & PADDR_MASK)
    }

    pub const fn vaddr<const O: usize>(&self) -> VirtAddr<O> {
        crate::addr::phys_to_virt(self.paddr())
    }

    pub const fn is_table_entry<L: TableLevel>(&self) -> bool {
        if !self.is_valid() {
            return false;
        }

        return L::LEVEL != 1 && (self.0 & 1 << TABLE_OFFSET == 1 << TABLE_OFFSET);
    }

    #[inline(always)]
    pub const fn page_entry<L: TableLevel>(
        paddr: PhysAddr,
        uxn: bool,
        global: bool,
        af: bool,
        share: Shareability,
        ap: AccessPermission,
        attr: MemoryAttr,
    ) -> Self {
        if L::LEVEL == 1 {
            Self(
                ((paddr.0 & PADDR_MASK)
                | (uxn as usize) << UXN_OFFSET         // universal execute never
                | ((!global) as usize) << N_G_OFFSET   // nG bit
                | (af as usize) << AF_OFFSET           // access flag
                | share as usize                       // Shareability
                | ap as usize                          // access permission
                | (attr as usize) << ATTR_INDEX_OFFSET // mair index
                | 1 << 1
                | 1 << VALID_OFFSET) as u64,
            )
        } else {
            Self(
                ((paddr.0 & PADDR_MASK)
                | (uxn as usize) << UXN_OFFSET         // universal execute never
                | ((!global) as usize) << N_G_OFFSET   // nG bit
                | (af as usize) << AF_OFFSET           // access flag
                | share as usize                       // Shareability
                | ap as usize                          // access permission
                | (attr as usize) << ATTR_INDEX_OFFSET // mair index
                | 1 << VALID_OFFSET) as u64,
            )
        }
    }

    pub fn normal_page_entry<L: TableLevel>(paddr: PhysAddr, perm: Permission) -> Self {
        let is_executable = perm.is_executable();
        Self::page_entry::<L>(
            paddr,
            !is_executable,
            false,
            true,
            Shareability::InnerSharable,
            perm.into(),
            MemoryAttr::Normal,
        )
    }

    pub fn device_page_entry<L: TableLevel>(paddr: PhysAddr, perm: Permission) -> Self {
        Self::page_entry::<L>(
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

impl PageTableEntry for Aarch64PageTableEntry {
    fn invalid_entry<L: TableLevel>() -> Self {
        return Self(0);
    }

    fn is_valid<L: TableLevel>(&self) -> bool {
        self.is_valid()
    }

    fn paddr<L: TableLevel>(&self) -> PhysAddr {
        self.paddr()
    }

    fn is_table_entry<L: TableLevel>(&self) -> bool {
        self.is_table_entry::<L>()
    }
}
