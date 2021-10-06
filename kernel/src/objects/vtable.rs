use super::*;
use core::convert::TryFrom;

use vspace::{Level2, Level3, Level4, PhysAddr, Table, TableLevel, VSpace, VirtAddr};

/* Capability Entry Field Definition
 * -------------------------------------------------
 * |                  paddr        |      recv     |
 * |                               |       12      |
 * -------------------------------------------------
 * |   ASID  |     mapped_vaddr    |      recv     |
 * |    16   |         36          |       12      |
 * -------------------------------------------------
 */
pub struct VTableObj([()]); // Make a RamObj not Sized

pub type VTableCap<'a> = CapRef<'a, VTableObj>;
impl<'a> VTableCap<'a> {
    pub fn mint(paddr: usize) -> CapRaw {
        CapRaw::new(paddr, 0, 0, None, None, ObjType::VTable)
    }

    pub fn set_mapped_vaddr_asid(&self, vaddr: usize, asid: usize, level: usize) {
        let mut raw = self.raw();
        raw.arg1 = (asid << 48) | (vaddr & MASK!(48));
        raw.arg2 = level;
        self.raw.replace(raw);
    }

    pub fn mapped_vaddr(&self) -> usize {
        self.raw().arg1 & MASK!(48)
    }

    pub fn mapped_asid(&self) -> usize {
        self.raw().arg1 >> 48
    }

    pub fn mapped_level(&self) -> usize {
        self.raw().arg2
    }

    pub fn debug_formatter(f: &mut core::fmt::DebugStruct, cap: &CapRaw) {
        let c = Cell::new(*cap);
        let c = VTableCap::try_from(&c).unwrap();
        f.field("vaddr", &c.vaddr());
    }

    pub fn map_vtable(
        &self,
        vspace: &mut VSpace<KERNEL_OFFSET>,
        vaddr: VirtAddr<KERNEL_OFFSET>,
        level: usize,
    ) -> SysResult<()> {
        match level {
            4 => vspace
                .map_table::<Level4>(vaddr, PhysAddr(self.paddr()))
                .map_err(|e| e.into()),
            3 => vspace
                .map_table::<Level3>(vaddr, PhysAddr(self.paddr()))
                .map_err(|e| e.into()),
            2 => vspace
                .map_table::<Level2>(vaddr, PhysAddr(self.paddr()))
                .map_err(|e| e.into()),
            _ => Err(SysError::InvalidValue),
        }?;

        let asid = (vspace.root_paddr().0 >> 12) & MASK!(16);
        self.set_mapped_vaddr_asid(vaddr.0, asid, level - 1);

        Ok(())
    }

    pub fn derive(&self, dst: &NullCap) -> SysResult<()> {
        dst.raw.set(self.raw());
        Ok(())
    }

    pub fn identify(&self, tcb: &mut TcbObj) -> usize {
        tcb.set_mr(1, self.cap_type() as usize);
        tcb.set_mr(2, self.mapped_vaddr());
        tcb.set_mr(3, self.mapped_asid());
        tcb.set_mr(4, self.mapped_level());
        4
    }

    pub fn init(&self) {}

    pub fn as_table<L: TableLevel>(&self) -> &Table<L, KERNEL_OFFSET> {
        unsafe { &*(self.vaddr() as *const Table<L, KERNEL_OFFSET>) }
    }

    pub fn as_table_mut<L: TableLevel>(&self) -> &mut Table<L, KERNEL_OFFSET> {
        unsafe { &mut *(self.vaddr() as *mut Table<L, KERNEL_OFFSET>) }
    }
}
