use crate::prelude::*;
use sysapi::error::{SysError, SysResult};
use core::ops::{Index, IndexMut};

macro_rules! pgd_index { ($x:expr) => (($x >> 39) & MASK!(9)) }
macro_rules! pud_index { ($x:expr) => (($x >> 30) & MASK!(9)) }
macro_rules! pd_index  { ($x:expr) => (($x >> 21) & MASK!(9)) }
macro_rules! pt_index  { ($x:expr) => (($x >> 12) & MASK!(9)) }

pub use sysapi::vspace::{FRAME_BIT_SIZE, FRAME_SIZE as PAGE_SIZE};

pub const PADDR_MASK : usize = MASK!(48) | (!MASK!(12));
const VALID_OFFSET: usize = 0;
const TABLE_OFFSET: usize = 1;
const UXN_OFFSET : usize = 54;
const N_G_OFFSET: usize  = 11;
const AF_OFFSET: usize  = 10;
const ATTR_INDEX_OFFSET: usize = 2;

#[allow(non_camel_case_types)]
pub enum MairFlag {
    Normal       = 0xff,
    Normal_NC    = 0x44,
    DevicenGnRnE = 0x00,
    DevicenGnRE  = 0x04,
    DeviceGRE    = 0x0c,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub enum MemoryAttr {
    Normal        = 0,
    Normal_NC     = 1,
    DevicenGnRnE  = 2,
    DevicenGnRE   = 3,
    DeviceGRE     = 4,
}

const AP_OFFSET: usize = 6;
#[derive(Copy, Clone, Debug)]
pub enum AccessPermission {
    KernelOnly = 0b00 << AP_OFFSET,
    ReadWrite  = 0b01 << AP_OFFSET,
    KernelRead = 0b10 << AP_OFFSET,
    ReadOnly   = 0b11 << AP_OFFSET
}

const SH_OFFSET: usize = 8;
#[derive(Copy, Clone, Debug)]
pub enum Shareability {
    NonSharable   = 0b00 << SH_OFFSET,
    Unpredictable = 0b01 << SH_OFFSET,
    OuterSharable = 0b10 << SH_OFFSET,
    InnerSharable = 0b11 << SH_OFFSET,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Entry (usize);

impl Entry {

    pub const fn zero() -> Self { Self(0) }

    pub const fn _new(entry: usize) -> Self {
        Self(entry)
    }

    #[inline(always)]
    pub const fn table_entry(paddr: usize) -> Self {
        Self((paddr & PADDR_MASK) | 1 << TABLE_OFFSET | 1 << VALID_OFFSET)
    }

    #[inline(always)]
    pub const fn block_entry(paddr: usize, uxn: bool, global: bool, af: bool, share: Shareability, ap: AccessPermission, attr: MemoryAttr) -> Self {
        Self((paddr & PADDR_MASK) 
             | (uxn as usize) << UXN_OFFSET         // universal execute never
             | ((!global) as usize) << N_G_OFFSET   // nG bit
             | (af as usize) << AF_OFFSET           // access flag
             | share as usize                       // Shareability
             | ap as usize                          // access permission
             | (attr as usize) << ATTR_INDEX_OFFSET // mair index
             | 1 << VALID_OFFSET)
    }

    #[inline(always)]
    pub const fn page_entry(paddr: usize, uxn: bool, global: bool, af: bool, share: Shareability, ap: AccessPermission, attr: MemoryAttr) -> Self {
        Self((paddr & PADDR_MASK) 
             | (uxn as usize) << UXN_OFFSET         // universal execute never
             | ((!global) as usize) << N_G_OFFSET   // nG bit
             | (af as usize) << AF_OFFSET           // access flag
             | share as usize                       // Shareability
             | ap as usize                          // access permission
             | (attr as usize) << ATTR_INDEX_OFFSET // mair index
             | 1 << 1
             | 1 << VALID_OFFSET)
    }

    pub const fn is_valid(&self) -> bool {
        self.0 & 0x1 == 0x1
    }

    pub const fn is_invalid(&self) -> bool {
        !self.is_valid()
    }

    pub fn into_table(&self) -> &mut Table {
        unsafe { &mut *((((self.0 & PADDR_MASK) & !MASK!(2)) + KERNEL_OFFSET) as *mut Table) }
    }
}

#[derive(Copy, Clone)]
#[repr(align(4096))]
pub struct Table {
    pub entries: [Entry; 512]
}

impl core::default::Default for Table {
    fn default() -> Self {
        Self::zero()
    }
}

impl Index<usize> for Table {
    type Output = Entry;
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for Table {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl core::fmt::Debug for Table {
    fn fmt(&self, _f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Ok(())
    }
}

impl Table {
    pub const fn zero() -> Self {
        Self {entries: [Entry::zero(); 512]}
    }

    pub fn paddr(&self) -> usize {
        self as *const _ as usize - KERNEL_OFFSET
    }
}

#[derive(Copy, Clone, Debug)]
pub struct VSpace {
    pub root: *mut Table
}

impl VSpace {
    pub fn as_addr(&self) -> usize {
        self.root as usize
    }

    pub fn from_pgd(pgd: &Table) -> Self {
        Self { root: pgd as *const Table as *mut Table }
    }

    pub fn from_paddr(paddr: usize) -> Self {
        let ptr = paddr + KERNEL_OFFSET;
        Self { root: ptr as *mut Table }
    }

    pub fn lookup_pgd_slot(&self, vaddr: usize) -> SysResult<&mut Entry> {
        let table = unsafe{ &mut *self.root };
        Ok(&mut table[pgd_index!(vaddr)])
    }

    pub fn lookup_pud_slot(&self, vaddr: usize) -> SysResult<&mut Entry> {
        let pgd_slot = self.lookup_pgd_slot(vaddr)?;
        if pgd_slot.is_invalid() {
            return Err(SysError::VSpaceTableMiss{ level: 2 });
        }
        Ok(&mut pgd_slot.into_table()[pud_index!(vaddr)])
    }

    pub fn lookup_pd_slot(&self, vaddr: usize) -> SysResult<&mut Entry> {
        let pud_slot = self.lookup_pud_slot(vaddr)?;
        if pud_slot.is_invalid() {
            return Err(SysError::VSpaceTableMiss{ level: 3 });
        }
        Ok(&mut pud_slot.into_table()[pd_index!(vaddr)])
    }

    pub fn lookup_pt_slot(&self, vaddr: usize) -> SysResult<&mut Entry> {
        let pd_slot = self.lookup_pd_slot(vaddr)?;
        if pd_slot.is_invalid() {
            return Err(SysError::VSpaceTableMiss { level: 4 });
        }
        Ok(&mut pd_slot.into_table()[pt_index!(vaddr)])
    }

    pub fn map_pud_table(&self, vaddr: usize, entry: Entry) -> SysResult<()> {
        let pgd_slot = self.lookup_pgd_slot(vaddr)?;
        if pgd_slot.is_valid() {
            return Err(SysError::VSpaceSlotOccupied{ level: 2 });
        }
        *pgd_slot = entry;
        crate::arch::dc_clean_by_va_PoU(pgd_slot as *const _ as usize);
        Ok(())
    }

    pub fn map_pd_table(&self, vaddr: usize, entry: Entry) -> SysResult<()> {
        let pud_slot = self.lookup_pud_slot(vaddr)?;
        if pud_slot.is_valid() {
            return Err(SysError::VSpaceSlotOccupied{ level: 3 });
        }
        *pud_slot = entry;
        crate::arch::dc_clean_by_va_PoU(pud_slot as *const _ as usize);
        Ok(())
    }

    pub fn map_pt_table(&self, vaddr: usize, entry: Entry) -> SysResult<()> {
        let pd_slot = self.lookup_pd_slot(vaddr)?;
        if pd_slot.is_valid() {
            return Err(SysError::VSpaceSlotOccupied{ level: 4 });
        }
        *pd_slot = entry;
        crate::arch::dc_clean_by_va_PoU(pd_slot as *const _ as usize);
        Ok(())
    }

    pub fn map_frame(&self, vaddr: usize, entry: Entry) -> SysResult<()> {
        let pt_slot = self.lookup_pt_slot(vaddr)?;
        if pt_slot.is_valid() {
            return Err(SysError::VSpaceSlotOccupied{ level: 5 });
        }
        *pt_slot = entry;
        crate::arch::dc_clean_by_va_PoU(pt_slot as *const _ as usize);
        Ok(())
    }

    pub fn asid(&self) -> usize {
        ((self.as_addr() - crate::prelude::KERNEL_OFFSET) >> 12) & MASK!(16)
    }
}

use super::{dsb, isb};

#[inline(always)]
pub unsafe fn set_mair(mair: usize) {
    llvm_asm!("msr mair_el1, $0"
         :
         : "r"(mair)
         : "memory");
    isb();
}

#[inline(always)]
pub unsafe fn enable_mmu(pgd_higher: usize)
{
    let mair_value = 
          (MairFlag::Normal as usize) << (MemoryAttr::Normal as usize * 8)
        | (MairFlag::Normal_NC as usize) << (MemoryAttr::Normal_NC as usize * 8)
        | (MairFlag::DevicenGnRnE as usize) << (MemoryAttr::DevicenGnRnE as usize * 8)
        | (MairFlag::DevicenGnRE as usize) << (MemoryAttr::DevicenGnRE as usize * 8)
        | (MairFlag::DeviceGRE as usize) << (MemoryAttr::DeviceGRE as usize * 8);
    set_mair(mair_value);

    install_kernel_vspace(pgd_higher);
    flush_tlb_allel1_is();
}


#[inline(always)]
pub unsafe fn install_kernel_vspace(paddr: usize)
{
    dsb();
    llvm_asm!("msr     ttbr1_el1, $0"
        :
        : "r"(paddr)
        : "memory"
    );
    isb();
}

#[inline(always)]
pub unsafe fn install_user_vspace(asid: usize, pgd: usize)
{
    let entry = asid << 48 | (pgd & MASK!(48));
    dsb();
    llvm_asm!("msr     ttbr0_el1, $0"
        :
        : "r"(entry)
        : "memory"
        : "volatile"
    );
    isb();
}

//#[inline(always)]
//#[allow(dead_code)]
//pub fn get_current_user_vspace() -> usize {
//    let ret;
//    unsafe {
//        llvm_asm!("mrs $0, ttbr0_el1": "=r"(ret));
//    }
//    ret
//}

#[inline(always)]
pub unsafe fn flush_tlb_allel1_is()
{
    dsb();
    llvm_asm!("
        dsb ishst
        tlbi vmalle1is
        dsb ish
    ");
    isb();
}

pub fn invalidateLocalTLB_ASID(asid: usize) {
    dsb();
    unsafe{ llvm_asm!("tlbi aside1, $0"::"r"(asid)) }
    dsb();
    isb();
}
