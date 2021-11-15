use core::cell::Cell;
use core::convert::TryFrom;
use core::mem::size_of;
use sysapi::objects::CNODE_DEPTH;

use super::*;

#[derive(Debug, Clone, Copy)]
pub enum CNodeLookupErr {
    CNodeMiss(usize),
    GuardError,
}

impl From<CNodeLookupErr> for SysError {
    fn from(_: CNodeLookupErr) -> Self {
        SysError::LookupError
    }
}

/* CNodeObj Field Definition
 * -----------------------------------------------
 * |            addr           |radix_sz|guard_sz|
 * |                           |   6    |   6    |
 * |---------------------------------------------|
 * |              cnode guard                    |
 * |                                             |
 * -----------------------------------------------
 */

pub type CNodeEntry = Cell<CapRaw>;

pub type CNodeObj = [CNodeEntry];

/* Asserting size of a CNodeEntry aligns power of 2 */
const_assert_eq!(
    size_of::<CNodeEntry>(),
    size_of::<CNodeEntry>().next_power_of_two()
);

pub const CNODE_ENTRY_SZ: usize = size_of::<CNodeEntry>().next_power_of_two();
pub const CNODE_ENTRY_BIT_SZ: usize = CNODE_ENTRY_SZ.trailing_zeros() as usize;
const_assert_eq!(
    CNODE_ENTRY_BIT_SZ,
    crate::objects::cnode::CNODE_ENTRY_BIT_SZ
);

pub type CNodeCap<'a> = CapRef<'a, CNodeObj>;

impl<'a> CNodeCap<'a> {
    const GUARD_SZ_OFFSET: usize = 0;
    const GUARD_SZ_BITS: usize = 4;
    const RADIX_SZ_OFFSET: usize = Self::GUARD_SZ_OFFSET + Self::GUARD_SZ_BITS;
    const RADIX_SZ_BITS: usize = 6;
    //    const ADDR_OFFSET    : usize = Self::RADIX_OFFSET + Self::RADIX_SZ_BITS;
    //    pub const ADDR_MASK      : usize = !MASK!(15);
    pub fn mint(paddr: usize, radix_sz: usize, guard_sz: usize, guard: usize) -> CapRaw {
        CapRaw::new(
            paddr,
            (guard_sz & MASK!(Self::GUARD_SZ_BITS)) << Self::GUARD_SZ_OFFSET
                | (radix_sz & MASK!(Self::RADIX_SZ_BITS)) << Self::RADIX_SZ_OFFSET,
            guard,
            None,
            None,
            ObjType::CNode,
        )
    }

    pub fn mint_from_entries(
        entries: &[CNodeEntry],
        guard: usize,
        guard_sz: usize,
    ) -> Option<CapRaw> {
        let paddr = entries.as_ptr() as usize - KERNEL_OFFSET;
        let radix_sz = entries.len().next_power_of_two().trailing_zeros() as usize;
        if radix_sz + guard_sz > CNODE_DEPTH {
            return None;
        }

        Some(Self::mint(paddr, radix_sz, guard_sz, guard))
    }

    pub fn as_object(&self) -> &[CNodeEntry] {
        use core::slice::from_raw_parts;

        unsafe { from_raw_parts(self.vaddr() as *const CNodeEntry, self.size()) }
    }

    pub fn as_object_mut(&self) -> &'static mut [CNodeEntry] {
        use core::slice::from_raw_parts_mut;

        unsafe { from_raw_parts_mut(self.vaddr() as *mut CNodeEntry, self.size()) }
    }

    pub fn init(&self) {
        let node: &[CNodeEntry] = self.as_object();

        for slot in node {
            slot.swap(&CNodeEntry::default());
        }
    }

    pub fn radix_bits(&self) -> usize {
        (self.raw.get().arg1 >> Self::RADIX_SZ_OFFSET) & MASK!(Self::RADIX_SZ_BITS)
    }

    pub fn guard_bits(&self) -> usize {
        (self.raw.get().arg1 >> Self::GUARD_SZ_OFFSET) & MASK!(Self::GUARD_SZ_BITS)
    }

    pub fn guard(&self) -> usize {
        self.raw.get().arg2 & !MASK!(self.guard_bits())
    }

    pub fn size(&self) -> usize {
        1 << self.radix_bits()
    }

    // fn resolve_address(&self, idx: usize, depth: usize) -> Result<&CNodeEntry, CNodeLookupErr> {
    //     let mut cnode_slot = self.raw;
    //     let mut n_bits = depth;

    //     while let Ok(cnode) = CNodeCap::try_from(cnode_slot) {
    //         let radix_bits = cnode.radix_bits();
    //         let guard_bits = cnode.guard_bits();
    //         let level_bits = radix_bits + guard_bits;

    //         let guard = (idx >> ((n_bits - guard_bits) & !0usize )) & MASK!(guard_bits);
    //         if cnode.guard() != guard {
    //             return Err(CNodeLookupErr::GuardError);
    //         }

    //         if level_bits > n_bits {
    //             return Err(CNodeLookupErr::GuardError);
    //         }

    //         let offset = (idx >> (n_bits - level_bits)) & MASK!(radix_bits);
    //         let cap = unsafe { &*(&cnode.as_object()[offset] as *const CNodeEntry) };

    //         if n_bits <= level_bits {
    //             return Ok(cap);
    //         }

    //         n_bits -= level_bits;
    //         cnode_slot = cap;
    //     };

    //     Ok(cnode_slot)
    // }

    pub fn lookup_slot(&self, idx: usize) -> Result<&CNodeEntry, CNodeLookupErr> {
        // Ok(unsafe { &*(&self.as_object()[idx] as *const CNodeEntry) })
        let slot = self
            .as_object()
            .get(idx)
            .ok_or(CNodeLookupErr::CNodeMiss(idx))?;
        Ok(unsafe { &*(slot as *const CNodeEntry) })

        // self.resolve_address(idx, CNODE_DEPTH)
    }

    pub fn derive(&self, dst: &NullCap) -> SysResult<()> {
        dst.raw.set(self.raw());
        Ok(())
    }

    pub fn debug_formatter(f: &mut core::fmt::DebugStruct, cap: &CapRaw) {
        let c = Cell::new(*cap);
        let c = CNodeCap::try_from(&c).unwrap();
        f.field("vaddr", &c.vaddr())
            .field("radix", &c.radix_bits())
            .field("guard bits", &c.guard_bits())
            .field("guard", &c.guard());
    }

    pub fn identify(&self, tcb: &mut TcbObj) -> usize {
        tcb.set_mr(1, self.cap_type() as usize);
        tcb.set_mr(2, self.size());

        2
    }
}

impl<'a> core::ops::Index<usize> for CNodeCap<'a> {
    type Output = CNodeEntry;
    fn index(&self, index: usize) -> &Self::Output {
        self.lookup_slot(index).unwrap()
    }
}
