use alloc::collections::linked_list::LinkedList;

use rustyl4api::error::SysResult;
use rustyl4api::object::{Capability, RamCap, RamObj, VTableCap, VTableObj};
use rustyl4api::vspace::Permission;

#[derive(Debug)]
pub enum VSpaceEntry {
    Table(VTableNode),
    Frame(FrameNode),
}

impl VSpaceEntry {
    pub const fn new_table(cap: Capability<VTableObj>) -> Self {
        Self::Table(VTableNode::from_cap(cap))
    }

    pub const fn new_frame(cap: Capability<RamObj>) -> Self {
        Self::Frame(FrameNode::from_cap(cap))
    }
}

#[derive(Debug)]
pub enum VSpaceManError {
    SlotOccupied { level: usize },
    SlotTypeError { level: usize },
    PageTableMiss { level: usize },
}

#[derive(Debug)]
pub struct FrameNode {
    cap: Capability<RamObj>,
}

impl FrameNode {
    pub const fn from_cap(cap: Capability<RamObj>) -> Self {
        Self { cap: cap }
    }
}

#[derive(Debug)]
pub struct VTableNode {
    cap: Capability<VTableObj>,
    entry: LinkedList<(usize, VSpaceEntry)>,
}

impl VTableNode {
    pub const fn from_cap(cap: Capability<VTableObj>) -> Self {
        Self {
            cap: cap,
            entry: LinkedList::new(),
        }
    }

    pub fn try_install_entry(
        &mut self,
        cur_level: usize,
        entry: VSpaceEntry,
        vaddr: usize,
        dst_level: usize,
    ) -> Result<(), VSpaceManError> {
        if cur_level > 4 {
            panic!("unreacheable level!");
        }

        let idx = (((vaddr & MASK!(48)) >> 12) >> ((4 - cur_level) * 9)) & MASK!(9);
        let slot_entry = self.entry.iter_mut().find(|e| e.0 == idx);

        match (slot_entry, cur_level == dst_level) {
            (None, true) => {
                self.entry.push_back((idx, entry));
                Ok(())
            }
            (Some((_, VSpaceEntry::Table(t))), false) => {
                t.try_install_entry(cur_level + 1, entry, vaddr, dst_level)
            }
            (Some(_e), true) => Err(VSpaceManError::SlotOccupied { level: cur_level }),
            (Some((_, VSpaceEntry::Frame(_f))), false) => {
                Err(VSpaceManError::SlotTypeError { level: cur_level })
            }
            (None, false) => Err(VSpaceManError::PageTableMiss { level: cur_level }),
        }
    }
}

#[derive(Debug)]
pub struct VSpaceMan {
    root: VTableNode,
}

impl VSpaceMan {
    pub const fn new(root_cnode_slot: Capability<VTableObj>) -> Self {
        Self {
            root: VTableNode::from_cap(root_cnode_slot),
        }
    }

    #[allow(unused_variables)]
    pub fn install_entry(&self, entry: VSpaceEntry, vaddr: usize, level: usize) -> SysResult<()> {
        // self.root.try_install_entry(1, entry, vaddr, level)
        Ok(())
    }

    pub fn root_cap_slot(&self) -> usize {
        self.root.cap.slot
    }

    pub fn map_frame(
        &self,
        frame: RamCap,
        vaddr: usize,
        perm: Permission,
        level: usize,
    ) -> SysResult<()> {
        let entry = VSpaceEntry::new_frame(frame.clone());
        self.install_entry(entry, vaddr, level)?;
        frame.map(self.root_cap_slot(), vaddr, perm)
    }

    pub fn map_table(&self, table: VTableCap, vaddr: usize, level: usize) -> SysResult<()> {
        let entry = VSpaceEntry::new_table(table.clone());
        self.install_entry(entry, vaddr, level)?;
        table.map(self.root_cap_slot(), vaddr, level)
    }
}
