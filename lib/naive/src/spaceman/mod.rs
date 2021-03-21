
extern crate alloc;

use crate::objects::KernelObject;
use rustyl4api::vspace::Permission;

pub mod cspace_man;
pub mod utspace_man;
pub mod vmspace_man;
pub mod vspace_man;

use core::alloc::Layout;

use crate::objects::identify::IdentifyResult;
use crate::objects::{CNodeCap, Capability, RamCap, RamObj, VTableObj, VTableCap};

use vspace_man::{VSpaceManError, VSpaceEntry};

#[derive(Debug)]
pub struct SpaceManager {
    vspace_man: vspace_man::VSpaceMan,
    cspace_man: cspace_man::CSpaceMan,
    utspace_man: utspace_man::UntypedSpaceMan,
    vmspace_man: vmspace_man::VMSpaceMan,
}

impl SpaceManager {
    pub fn new(
        root_cnode: CNodeCap,
        root_cnode_size: usize,
        root_vnode: VTableCap,
    ) -> Self {
        Self {
            vspace_man: vspace_man::VSpaceMan::new(root_vnode),
            cspace_man: cspace_man::CSpaceMan::new(root_cnode, root_cnode_size),
            utspace_man: utspace_man::UntypedSpaceMan::new(),
            vmspace_man: vmspace_man::VMSpaceMan::new(),
        }
    }

    pub fn insert_untyped(
        &self,
        slot: usize,
        paddr: usize,
        bit_sz: u8,
        is_device: bool,
        free_offset: usize,
    ) {
        self.utspace_man
            .insert_untyped(slot, paddr, bit_sz, is_device, free_offset)
    }

    // pub fn insert_vm_range(&mut self, start: usize, end: usize) {
    //     self.vmspace_man.insert_vma(start, end);
    // }

    pub fn insert_identify(&self, slot: usize, result: IdentifyResult) {
        let slot = self.cspace_alloc_at(slot).unwrap();
        match result {
            IdentifyResult::NullObj => {}
            IdentifyResult::Untyped {
                paddr,
                bit_sz,
                is_device,
                free_offset,
            } => {
                // self.utspace_man.insert_untyped(slot, paddr, bit_sz, is_device, free_offset)
                self.insert_untyped(slot, paddr, bit_sz, is_device, free_offset);
            }
            IdentifyResult::CNode { bit_sz: _ } => {}
            IdentifyResult::Tcb => {}
            IdentifyResult::Ram {
                bit_sz: _,
                mapped_vaddr,
                mapped_asid: _,
                is_device: _,
            } => {
                let cap = RamCap::new(slot);
                self.vspace_man.map_frame(cap, mapped_vaddr, Permission::writable(), 4, false).unwrap();
            }
            IdentifyResult::VTable {
                mapped_vaddr,
                mapped_asid: _,
                level,
            } if level > 1 => {
                let cap= Capability::<VTableObj>::new(slot);
                self.vspace_man.map_table(cap, mapped_vaddr, level - 1, false).unwrap();
            }
            IdentifyResult::Endpoint => {}
            IdentifyResult::Reply => {}
            IdentifyResult::Monitor => {}
            IdentifyResult::Interrupt => {}
            _ => {}
        }
    }

    pub fn cspace_alloc_at(&self, slot: usize) -> Option<usize> {
        self.cspace_man.allocate_slot_at(slot)
    }

    pub fn cspace_alloc(&self) -> Option<usize> {
        self.cspace_man.allocate_slot()
    }

    pub fn cspace_free(&self, slot: usize) {
        self.cspace_man.free_slot(slot)
    }

    pub fn vspace_alloc(&self, layout: Layout) -> Option<usize> {
        Some(self.vmspace_man.allocate_mem(layout))
    }

    pub fn map_frame_at(
        &self,
        paddr: usize,
        vaddr: usize,
        size: usize,
        perm: Permission,
    ) -> Result<*mut u8, ()> {
        if paddr != 0 {
            return Err(());
        }

        let mut rem_size = crate::utils::align_up(size, 4096);
        let base_vaddr = if vaddr == 0 {
            let layout = Layout::from_size_align(rem_size, 4096).unwrap();
            self.vspace_alloc(layout).unwrap()
        } else {
            vaddr
        };
        let mut vaddr = base_vaddr;

        while rem_size > 0 {
            let frame = self.alloc_object::<RamObj>(12).ok_or(())?;
            self.insert_ram_at(frame, vaddr, perm);
            vaddr += 4096;
            rem_size -= 4096;
        }

        Ok(base_vaddr as *mut u8)
    }

    /// Insert an RamCap to vspace to manage and handle backed page table
    pub fn insert_ram_at(&self, ram: RamCap, vaddr: usize, perm: Permission) -> *mut u8 {
        // TODO: support large page
        let layout = Layout::from_size_align(4096, 4096).unwrap();
        let vaddr = if vaddr == 0 {
            self.vspace_alloc(layout).unwrap()
        } else {
            vaddr
        };
        let mut frame_entry = VSpaceEntry::new_frame(ram, vaddr, perm, 4);
        loop {
            let res = self.vspace_man.install_entry(frame_entry, true);
            if let Err((e, ent)) = res {
                frame_entry = ent;
                match e {
                    // VSpaceManError::SlotOccupied{level} => {
                    //     panic!("slot occupied at level {} vaddr {:x}", level, vaddr);
                    // }
                    // VSpaceManError::SlotTypeError{level} => {
                    //     panic!("wrong slot type at level {} vaddr {:x}", level, vaddr);
                    // }
                    VSpaceManError::PageTableMiss { level } => {
                        let vtable_cap = self.alloc_object::<VTableObj>(12).unwrap();
                        self.vspace_man
                            .map_table(vtable_cap, vaddr, level + 1, true)
                            .unwrap();
                    }
                    e => {
                        panic!("unexpected error {:?}", e);
                    }
                }

            } else {
                break;
            }
        }
        vaddr as *mut u8
    }

    pub fn insert_vtable(&self, table: VTableCap, vaddr: usize, level: usize, do_map: bool) {
        let entry = vspace_man::VSpaceEntry::new_table(table, vaddr, level);
        self.vspace_man.install_entry(entry, do_map).unwrap();
    }

    pub fn install_ram(&self, ram: RamCap, vaddr: usize, perm: Permission, level: usize, do_map:bool) {
        let entry = vspace_man::VSpaceEntry::new_frame(ram, vaddr, perm, level);
        self.vspace_man.install_entry(entry, do_map).unwrap();
    }

    pub fn alloc_object<T: KernelObject>(&self, size: usize) -> Option<Capability<T>> {
        let slot = self.cspace_alloc()?;
        self.utspace_man.alloc_object::<T>(slot, size)
    }

    //    pub fn alloc_object_at<T: KernelObject>(&self, paddr: usize, bit_sz: usize, maybe_device: bool) -> Option<Capability<RamObj>> {
    //        unimplemented!();
    //    }
}
