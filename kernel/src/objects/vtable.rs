use super::*;
use core::convert::TryFrom; 
use crate::vspace::Table;
use crate::arch::vspace::{Entry, VSpace};

/* Capability Entry Field Definition
 * -------------------------------------------------
 * |                  paddr        |      recv     |
 * |                               |       12      |
 * -------------------------------------------------
 * |   ASID  |     mapped_vaddr    |      recv     |
 * |    16   |         36          |       12      |
 * -------------------------------------------------
 */
pub type VTableObj = Table;

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

    pub fn map_vtable(&self, vspace: &VSpace, vaddr: usize, level: usize) -> SysResult<()> {

        let entry = Entry::table_entry(self.paddr());

        match level {
            2 => vspace.map_pud_table(vaddr, entry),
            3 => vspace.map_pd_table(vaddr, entry),
            4 => vspace.map_pt_table(vaddr, entry),
            _ => Err(SysError::InvalidValue)
        }?;

        self.set_mapped_vaddr_asid(vaddr, vspace.asid(), level);

        Ok(())
    }

    pub fn derive(&self, dst: &NullCap) -> SysResult<()>{
        dst.raw.set(self.raw());
        Ok(())
    }

    pub fn identify(&self, tcb: &TcbObj) -> usize {
        tcb.set_mr(1, self.cap_type() as usize);
        tcb.set_mr(2, self.mapped_vaddr());
        tcb.set_mr(3, self.mapped_asid());
        tcb.set_mr(4, self.mapped_level());
        4
    }
}