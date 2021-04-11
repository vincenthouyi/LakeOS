use alloc::collections::linked_list::LinkedList;

use spin::Mutex;

use rustyl4api::error::SysResult;
use crate::objects::{RamRef, VTableRef};
use rustyl4api::vspace::Permission;

#[derive(Debug)]
pub enum VSpaceEntry {
    Table(VTableNode),
    Frame(FrameNode),
}

impl VSpaceEntry {
    pub const fn new_table(cap: VTableRef, vaddr: usize, level: usize) -> Self {
        Self::Table(VTableNode::new(cap, vaddr, level))
    }

    pub const fn new_frame(cap: RamRef, vaddr: usize, perm: Permission, level: usize) -> Self {
        Self::Frame(FrameNode::new(cap, vaddr, perm, level))
    }

    pub fn is_table(&self) -> bool {
        if let VSpaceEntry::Table(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_vtable_node(&self) -> Option<&VTableNode> {
        if let VSpaceEntry::Table(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn as_vtable_node_mut(&mut self) -> Option<&mut VTableNode> {
        if let VSpaceEntry::Table(node) = self {
            Some(node)
        } else {
            None
        }
    }

    pub fn into_vtablecap(self) -> Result<VTableRef, Self> {
        if let VSpaceEntry::Table(node) = self {
            Ok(node.cap)
        } else {
            Err(self)
        }
    }

    pub fn into_ramcap(self) -> Result<RamRef, Self> {
        if let VSpaceEntry::Frame(node) = self {
            Ok(node.cap)
        } else {
            Err(self)
        }
    }

    // pub fn cap_slot(&self) -> CapSlot {
    //     match self {
    //         VSpaceEntry::Frame(f) => f.cap.slot,
    //         VSpaceEntry::Table(t) => t.cap.slot,
    //     }
    // }

    pub fn vaddr(&self) -> usize {
        match self {
            VSpaceEntry::Table(t) => t.vaddr,
            VSpaceEntry::Frame(f) => f.vaddr,
        }
    }

    pub fn map_to_vspace(&self, root: &VTableRef) -> SysResult<()> {
        match self {
            VSpaceEntry::Frame(f) => f.map_to_vspace(root),
            VSpaceEntry::Table(t) => t.map_to_vspace(root),
        }
    }

    pub fn level(&self) -> usize {
        match self {
            VSpaceEntry::Frame(f) => f.level,
            VSpaceEntry::Table(t) => t.level,
        }

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
    vaddr: usize,
    cap: RamRef,
    perm: Permission,
    level: usize,
}

impl FrameNode {
    pub const fn new(cap: RamRef, vaddr: usize, perm: Permission, level: usize) -> Self {
        Self { vaddr, cap, perm, level }
    }

    pub fn map_to_vspace(&self, root: &VTableRef) -> SysResult<()> {
        self.cap
            .map(root, self.vaddr, self.perm)
    }
}

#[derive(Debug)]
pub struct VTableNode {
    vaddr: usize,
    cap: VTableRef,
    level: usize,
    entry: LinkedList<VSpaceEntry>,
}

fn vaddr_to_idx(vaddr: usize, level: usize) -> usize {
    (((vaddr & MASK!(48)) >> 12) >> ((3 - level) * 9)) & MASK!(9)
}

impl VTableNode {
    pub const fn new(cap: VTableRef, vaddr: usize, level: usize) -> Self {
        Self {
            vaddr,
            cap,
            level,
            entry: LinkedList::new(),
        }
    }

    pub fn lookup_entry(&mut self, vaddr: usize) -> Option<&mut VSpaceEntry> {
        let level = self.level;
        let idx = vaddr_to_idx(vaddr, level);
        self.entry
            .iter_mut()
            .find(|e| vaddr_to_idx(e.vaddr(), level) == idx)
    }

    pub fn insert_entry(&mut self, entry: VSpaceEntry) -> Result<&VSpaceEntry, (VSpaceManError, VSpaceEntry)> {
        let vaddr = entry.vaddr();

        if self.lookup_entry(vaddr).is_some() {
            return Err((VSpaceManError::SlotOccupied{ level: self.level }, entry))
        }
        self.entry.push_back(entry);
        Ok(self.entry.back_mut().unwrap())
    }

    pub fn map_to_vspace(&self, root: &VTableRef) -> SysResult<()> {
        self.cap
            .map(root, self.vaddr, self.level + 1)
    }
}

#[derive(Debug)]
struct VSpace(VSpaceEntry);

impl VSpace {
    pub const fn new(cap: VTableRef) -> Self {
        Self(VSpaceEntry::new_table(cap, 0, 0))
    }

    pub fn root_cap(&self) -> VTableRef {
        self.0.as_vtable_node().unwrap().cap.clone()
    }

    fn lookup_entry(&mut self, vaddr: usize, level: usize) -> Result<&mut VSpaceEntry, VSpaceManError> {
        let mut cur_level = 0;
        let mut cur_node = &mut self.0;
        while cur_level < level {
            cur_node = cur_node
                .as_vtable_node_mut()
                .ok_or(VSpaceManError::SlotTypeError{ level: cur_level })?
                .lookup_entry(vaddr)
                .ok_or(VSpaceManError::PageTableMiss{ level: cur_level })?;
            cur_level += 1;
        }
        Ok(cur_node)
    }

    pub fn install_entry(&mut self, entry: VSpaceEntry, do_map: bool) -> Result<(), (VSpaceManError, VSpaceEntry)> {
        let vaddr = entry.vaddr();
        let level = entry.level();
        let root_cap= self.root_cap();

        if level == 0 {
            todo!("install root entry");
        }

        let parent_entry = self.lookup_entry(vaddr, level - 1);
        if let Err(e) = parent_entry {
            return Err((e, entry));
        }
        let parent_entry = parent_entry.unwrap().as_vtable_node_mut();
        if let None = parent_entry {
            return Err((VSpaceManError::SlotTypeError { level: level - 1 }, entry));
        }
        let parent_entry = parent_entry.unwrap();
        let entry = parent_entry.insert_entry(entry)?;
        if do_map {
            entry.map_to_vspace(&root_cap).expect("failed to map to kernel");
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct VSpaceMan {
    root: Mutex<VSpace>,
}

impl VSpaceMan {
    pub const fn new(root_cnode_slot: VTableRef) -> Self {
        Self {
            root: Mutex::new(VSpace::new(root_cnode_slot)),
        }
    }

    pub fn install_entry(&self, entry: VSpaceEntry, do_map: bool) -> Result<(), (VSpaceManError, VSpaceEntry)> {
        self.root
            .lock()
            .install_entry(entry, do_map)
    }

    pub fn map_frame(
        &self,
        frame: RamRef,
        vaddr: usize,
        perm: Permission,
        level: usize,
        do_map: bool,
    ) -> Result<(), (VSpaceManError, RamRef)> {
        let entry = VSpaceEntry::new_frame(frame, vaddr, perm, level);
        self.install_entry(entry, do_map)
            .map_err(|(e, ent)| (e, ent.into_ramcap().unwrap()))
    }

    pub fn map_table(&self, table: VTableRef, vaddr: usize, level: usize, do_map: bool) -> Result<(), (VSpaceManError, VTableRef)> {
        let entry = VSpaceEntry::new_table(table, vaddr, level);
        self.install_entry(entry, do_map)
            .map_err(|(e, ent)| (e, ent.into_vtablecap().unwrap()))
    }

    pub fn memory_unmap(&self, base_ptr: *mut u8, len: usize) {
        //TODO
    }
}
