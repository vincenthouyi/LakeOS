use crate::PhysAddr;
use crate::permission::Permission;
use super::asm::*;

#[allow(non_camel_case_types)]
pub enum MairFlag {
    Normal = 0xff,
    NormalNC = 0x44,
    DevicenGnRnE = 0x00,
    DevicenGnRE = 0x04,
    DeviceGRE = 0x0c,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub enum MemoryAttr {
    Normal = 0,
    NormalNC = 1,
    DevicenGnRnE = 2,
    DevicenGnRE = 3,
    DeviceGRE = 4,
}

const AP_OFFSET: usize = 6;
#[derive(Copy, Clone, Debug)]
pub enum AccessPermission {
    KernelOnly = 0b00 << AP_OFFSET,
    ReadWrite = 0b01 << AP_OFFSET,
    KernelRead = 0b10 << AP_OFFSET,
    ReadOnly = 0b11 << AP_OFFSET,
}

impl Into<AccessPermission> for Permission {
    fn into(self) -> AccessPermission {
        if self.contains(Permission::READABLE & Permission::WRITABLE) {
            AccessPermission::ReadWrite
        } else if self.contains(Permission::READABLE) {
            AccessPermission::ReadOnly
        } else {
            AccessPermission::KernelOnly
        }
    }
}

const SH_OFFSET: usize = 8;
#[derive(Copy, Clone, Debug)]
pub enum Shareability {
    NonSharable = 0b00 << SH_OFFSET,
    Unpredictable = 0b01 << SH_OFFSET,
    OuterSharable = 0b10 << SH_OFFSET,
    InnerSharable = 0b11 << SH_OFFSET,
}


#[inline(always)]
pub unsafe fn init_mmu() {
    let mair_value = (MairFlag::Normal as usize) << (MemoryAttr::Normal as usize * 8)
        | (MairFlag::NormalNC as usize) << (MemoryAttr::NormalNC as usize * 8)
        | (MairFlag::DevicenGnRnE as usize) << (MemoryAttr::DevicenGnRnE as usize * 8)
        | (MairFlag::DevicenGnRE as usize) << (MemoryAttr::DevicenGnRE as usize * 8)
        | (MairFlag::DeviceGRE as usize) << (MemoryAttr::DeviceGRE as usize * 8);
    set_mair(mair_value);

    flush_tlb_allel1_is();
}

#[inline(always)]
pub unsafe fn install_kernel_vspace(paddr: PhysAddr) {
    dsb();
    llvm_asm!("msr     ttbr1_el1, $0"
        :
        : "r"(paddr.0)
        : "memory"
    );
    isb();
    flush_tlb_allel1_is();
}

#[inline(always)]
pub unsafe fn install_user_vspace(asid: usize, pgd: usize) {
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

pub fn invalidate_local_tlb_asid(asid: usize) {
    dsb();
    unsafe { llvm_asm!("tlbi aside1, $0"::"r"(asid)) }
    dsb();
    isb();
}
