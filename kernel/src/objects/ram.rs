use super::*;
use core::convert::TryFrom;
use crate::syscall::{MsgInfo, RespInfo, SyscallOp};
use sysapi::vspace::Permission;
use crate::vspace::{Entry, Shareability, MemoryAttr, AccessPermission, VSpace};

/* Capability Entry Field Definition
 * -------------------------------------------------
 * |   recv  |        paddr        |W|R|bit_sz|    |
 * |    16   |         36          |1|1|  6   | 4  |
 * -------------------------------------------------
 * |   ASID  |     mapped_vaddr    |      recv     |
 * |    16   |         36          |       12      |
 * -------------------------------------------------
 */
pub struct RamObj([()]); // Make a RamObj not Sized

pub type RamCap<'a> = CapRef<'a, RamObj>;

const ADDR_BITS : usize = 36;
const ADDR_OFFSET: usize = 12;
const WRITE_OFFSET: usize = 11;
//const WRITE_BITS: usize = 1;
const WRITE_MASK: usize = 0b100000000000;
//const READ_BITS: usize = 1;
const READ_OFFSET: usize = 10;
const READ_MASK: usize = 0b10000000000;
const BIT_SZ_BITS: usize = 6;
const BIT_SZ_OFFSET: usize = 4;

impl<'a> CapRef<'a, RamObj>{
    pub const ADDR_MASK: usize = MASK!(ADDR_BITS + ADDR_OFFSET) & !MASK!(ADDR_OFFSET);
    pub const fn mint(paddr: usize, writable: bool, readable: bool, bit_sz: usize, is_device: bool) -> CapRaw {
        CapRaw::new(paddr, 
                    ((writable as usize) << WRITE_OFFSET) 
                    | ((readable as usize) << READ_OFFSET) 
                    | ((bit_sz & MASK!(BIT_SZ_BITS)) << BIT_SZ_OFFSET),
                    is_device as usize,
                    None,
                    None,
                    ObjType::Ram)
    }

    pub fn is_writable(&self) -> bool {
        self.raw.get().arg1 & WRITE_MASK != 0
    }

    pub fn is_readable(&self) -> bool {
        self.raw.get().arg1 & READ_MASK != 0
    }

    pub fn is_device(&self) -> bool {
        self.raw.get().arg2 & 0b1 != 0
    }

    pub fn set_mapped_vaddr_asid(&self, vaddr: usize, asid: usize) {
        let mut raw = self.raw();
        raw.arg2 = asid << 48 | vaddr | raw.arg2 & MASK!(12);
        self.raw.replace(raw);
    }

    pub fn mapped_vaddr(&self) -> usize {
        self.raw().arg2 & Self::ADDR_MASK
    }

    pub fn mapped_asid(&self) -> usize {
        self.raw().arg2 >> 48
    }

    pub fn size(&self) -> usize {
        (self.raw().arg1 >> BIT_SZ_OFFSET) & MASK!(BIT_SZ_BITS)
    }

    pub fn debug_formatter(f: &mut core::fmt::DebugStruct, cap: &CapRaw) {
        let c = Cell::new(*cap);
        let c = RamCap::try_from(&c).unwrap();
        f.field("vaddr", &c.vaddr())
         .field("bit size", &c.size());
        return;
    }

    pub fn as_object(&self) -> &[u8] {
        use core::slice::from_raw_parts;

        unsafe { from_raw_parts(
                    self.vaddr() as *const u8,
                    1 << self.size()
        )}
    }

    pub fn as_object_mut(&mut self) -> &mut [u8] {
        use core::slice::from_raw_parts_mut;

        unsafe { from_raw_parts_mut(
                    self.vaddr() as *mut u8,
                    1 << self.size()
        )}
    }

    pub fn init(&mut self) {
        if !self.is_device() {
            for byte in self.as_object_mut() {
                *byte = 0u8;
            }
        }
    }

    pub fn map_page(&self, vspace: &VSpace, vaddr: usize, rights: Permission) -> SysResult<()> {

        let executable = rights.executable;
        let access = match (rights.readable, rights.writable) {
            (false, false) => {AccessPermission::KernelOnly}
            (false, true) => {return Err(SysError::VSpaceError);}
            (true, false) => {AccessPermission::ReadOnly}
            (true, true) => {AccessPermission::ReadWrite}
        };
        let mem_attr= match self.is_device() {
            true => MemoryAttr::DevicenGnRnE,
            false => MemoryAttr::Normal,
        };
        let share = match self.is_device() {
            true => Shareability::NonSharable,
            false => Shareability::InnerSharable,
        };
        let entry = Entry::page_entry(self.paddr(), !executable, false, true, share,
                                    access, mem_attr);
        vspace.map_frame(vaddr, entry).or(Err(SysError::VSpaceError))?;

        self.set_mapped_vaddr_asid(vaddr, vspace.asid());

        Ok(())
    }

    pub fn handle_invocation(&self, info: MsgInfo, tcb: &mut TcbObj) -> SysResult<()> {

        match info.get_label() {
            SyscallOp::RamMap => {

                if self.mapped_vaddr() != 0 {
                    return Err(SysError::VSpaceError)
                }

                if info.get_length() < 3 {
                    return Err(SysError::InvalidValue);
                }

                let vspace_cap_idx = tcb.get_mr(1);
                let vaddr = tcb.get_mr(2);
                let rights = tcb.get_mr(3).into();
                let cspace = tcb.cspace().unwrap();

                let vspace_cap_slot = cspace.lookup_slot(vspace_cap_idx)?;
                let vspace = VSpace::from_pgd(&*(VTableCap::try_from(vspace_cap_slot)?));

                self.map_page(&vspace, vaddr, rights)?;

                tcb.set_respinfo(RespInfo::new(SysError::OK, 0));
                Ok(())
            }
            SyscallOp::CapIdentify => {
                tcb.set_mr(1, self.cap_type() as usize);
                tcb.set_mr(2, self.size());
                tcb.set_mr(3, self.mapped_vaddr());
                tcb.set_mr(4, self.mapped_asid());
                tcb.set_mr(5, self.is_device() as usize);

                tcb.set_respinfo(RespInfo::new(SysError::OK, 1));

                Ok(())
            }
            _ => { Err(SysError::UnsupportedSyscallOp) }
        }
    }
}

impl<'a> core::fmt::Debug for CapRef<'a, RamObj> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Ram Capability")
         .field("paddr", &self.paddr())
         .field("writable", &self.is_writable())
         .field("readdable", &self.is_readable())
         .field("size bits", &self.size())
         .finish()
    }
}

impl<'a> core::ops::Deref for RamCap<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        use core::slice::from_raw_parts;
        unsafe { from_raw_parts(self.vaddr() as *const u8, 4096) }
    }
}

impl<'a> core::ops::DerefMut for RamCap<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        use core::slice::from_raw_parts_mut;
        unsafe { from_raw_parts_mut(self.vaddr() as *mut u8, 4096) }
    }
}
