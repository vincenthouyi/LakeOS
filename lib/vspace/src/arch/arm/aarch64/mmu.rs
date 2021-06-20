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
        if self == Permission::READWIRTE {
            AccessPermission::ReadWrite
        } else if self == Permission::READONLY {
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


// #[inline(always)]
// pub unsafe fn enable_mmu(pgd_higher: usize) {
//     let mair_value = (MairFlag::Normal as usize) << (MemoryAttr::Normal as usize * 8)
//         | (MairFlag::NormalNC as usize) << (MemoryAttr::NormalNC as usize * 8)
//         | (MairFlag::DevicenGnRnE as usize) << (MemoryAttr::DevicenGnRnE as usize * 8)
//         | (MairFlag::DevicenGnRE as usize) << (MemoryAttr::DevicenGnRE as usize * 8)
//         | (MairFlag::DeviceGRE as usize) << (MemoryAttr::DeviceGRE as usize * 8);
//     set_mair(mair_value);

//     install_kernel_vspace(pgd_higher);
//     flush_tlb_allel1_is();
// }
