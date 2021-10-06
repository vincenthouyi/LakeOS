use super::*;
use core::convert::TryFrom;
use sysapi::vspace::Permission;
use vspace::{Entry, Level1, Level4, PageLevel, PhysAddr, Table, VSpace, VirtAddr};

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

const ADDR_BITS: usize = 36;
const ADDR_OFFSET: usize = 12;
const WRITE_OFFSET: usize = 11;
//const WRITE_BITS: usize = 1;
const WRITE_MASK: usize = 0b100000000000;
//const READ_BITS: usize = 1;
const READ_OFFSET: usize = 10;
const READ_MASK: usize = 0b10000000000;
const BIT_SZ_BITS: usize = 6;
const BIT_SZ_OFFSET: usize = 4;

impl<'a> CapRef<'a, RamObj> {
    pub const ADDR_MASK: usize = MASK!(ADDR_BITS + ADDR_OFFSET) & !MASK!(ADDR_OFFSET);
    pub const fn mint(
        paddr: usize,
        writable: bool,
        readable: bool,
        bit_sz: usize,
        is_device: bool,
    ) -> CapRaw {
        CapRaw::new(
            paddr,
            ((writable as usize) << WRITE_OFFSET)
                | ((readable as usize) << READ_OFFSET)
                | ((bit_sz & MASK!(BIT_SZ_BITS)) << BIT_SZ_OFFSET),
            is_device as usize,
            None,
            None,
            ObjType::Ram,
        )
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
        f.field("vaddr", &c.vaddr()).field("bit size", &c.size());
        return;
    }

    pub fn as_object(&self) -> &[u8] {
        use core::slice::from_raw_parts;

        unsafe { from_raw_parts(self.vaddr() as *const u8, 1 << self.size()) }
    }

    pub fn as_object_mut(&mut self) -> &mut [u8] {
        use core::slice::from_raw_parts_mut;

        unsafe { from_raw_parts_mut(self.vaddr() as *mut u8, 1 << self.size()) }
    }

    pub fn init(&mut self) {
        if !self.is_device() {
            for byte in self.as_object_mut() {
                *byte = 0u8;
            }
        }
    }

    pub fn map_page<L: PageLevel>(
        &self,
        vspace: &mut VSpace<KERNEL_OFFSET>,
        vaddr: usize,
        rights: Permission,
    ) -> SysResult<()> {
        if self.is_device() {
            vspace.map_device_frame::<L>(
                VirtAddr(vaddr),
                PhysAddr::<KERNEL_OFFSET>(self.paddr()),
                rights,
            )?
        } else {
            vspace.map_normal_frame::<L>(
                VirtAddr(vaddr),
                PhysAddr::<KERNEL_OFFSET>(self.paddr()),
                rights,
            )?
        }

        let asid = (vspace.root_paddr().0 >> 12) & MASK!(16);
        self.set_mapped_vaddr_asid(vaddr, asid);

        Ok(())
    }

    pub fn unmap_page(&self) -> SysResult<()> {
        let asid = self.mapped_asid();
        let root_table =
            unsafe { &mut *(((asid << 12) + KERNEL_OFFSET) as *mut Table<Level4, KERNEL_OFFSET>) };
        let mut vspace = VSpace::from_root(root_table);
        let mapped_vaddr = self.mapped_vaddr();

        let slot = vspace.lookup_slot_mut::<Level1>(VirtAddr(mapped_vaddr))?;
        *slot = Entry::zero();

        crate::arch::dc_clean_by_va_PoU(slot as *const _ as usize);
        crate::arch::dmb();

        Ok(())
    }

    pub fn identify(&self, tcb: &mut TcbObj) -> usize {
        tcb.set_mr(1, self.cap_type() as usize);
        tcb.set_mr(2, self.size());
        tcb.set_mr(3, self.mapped_vaddr());
        tcb.set_mr(4, self.mapped_asid());
        tcb.set_mr(5, self.is_device() as usize);
        5
    }

    pub fn derive(&self) -> CapRaw {
        Self::mint(
            self.paddr(),
            self.is_writable(),
            self.is_readable(),
            self.size(),
            self.is_device(),
        )
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
        self.as_object()
    }
}

impl<'a> core::ops::DerefMut for RamCap<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_object_mut()
    }
}
