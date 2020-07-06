#![feature(linked_list_cursors)]
#![feature(const_saturating_int_methods)]

#![no_std]

extern crate alloc;

use rustyl4api::object::KernelObject;

#[macro_use]mod utils {
    pub const fn align_down(addr: usize, align: usize) -> usize {
        addr & !(align - 1)
    }

    pub const fn align_up(addr: usize, align: usize) -> usize {
        align_down(addr.saturating_add(align - 1), align)
    }

    #[macro_export]
    macro_rules! MASK {
        ($x:expr) => (BIT!($x) - 1);
    }

    #[macro_export]
    macro_rules! BIT {
        ($x:expr) => (1 << $x);
    }
}
mod vspace_allocator;
mod cspace_man;
mod utspace_man;


use core::alloc::Layout;

use vspace_allocator::VspaceAllocator;

use rustyl4api::object::{Capability, CNodeObj, RamObj, VTableObj};
//use rustyl4api::object::identify::IdentifyResult;

pub struct SpaceManager {
    vspace_alloc: VspaceAllocator,
    cspace_man: cspace_man::CSpaceMan,
    utspace_man: utspace_man::UntypedSpaceMan,
}

impl SpaceManager {
    pub fn new(root_cnode: Capability<CNodeObj>, root_cnode_size: usize, root_vnode: Capability<VTableObj>, brk: usize) -> Self {
        Self {
            vspace_alloc: VspaceAllocator::new(root_vnode, brk),
            cspace_man: cspace_man::CSpaceMan::new(root_cnode, root_cnode_size),
            utspace_man: utspace_man::UntypedSpaceMan::new(),
        }
    }

    pub fn insert_untyped(&mut self, slot: usize, paddr: usize, bit_sz: u8, is_device: bool, free_offset: usize) {
        self.utspace_man
            .insert_untyped(slot, paddr, bit_sz, is_device, free_offset)
    }

//    pub fn insert_cap_from_identify(&mut self, slot: usize, result: IdentifyResult) {
//        match result {
//            IdentifyResult::NullObj => { },
//            IdentifyResult::Untyped {paddr, bit_sz, is_device, free_offset} => {
//                self.utspace_man.insert_untyped(slot, paddr, bit_sz, is_device, free_offset)
//            },
//            IdentifyResult::CNode {bit_sz} => { },
//            IdentifyResult::Tcb => { },
//            IdentifyResult::Ram {bit_sz, mapped_vaddr, mapped_asid, is_device} => {
//            },
//            IdentifyResult::VTable {mapped_vaddr, mapped_asid} => {
//            },
//            IdentifyResult::Endpoint => { },
//            IdentifyResult::Monitor => { },
//            IdentifyResult::Interrupt => { },
//        }
//    }

    pub fn cspace_alloc_at(&mut self, slot: usize) -> Option<usize> {
        self.cspace_man
            .allocate_slot_at(slot)
    }

    pub fn cspace_alloc(&mut self) -> Option<usize> {
        self.cspace_man
            .allocate_slot()
    }

    pub fn cspace_free(&mut self, slot: usize) {
        unimplemented!()
    }

    pub fn vspace_alloc(&mut self, layout: Layout) -> Option<usize> {
        Some(self.vspace_alloc
            .allocate(layout))
    }

    /// Insert an RamCap to vspace to manage and handle backed page table
    pub fn insert_ram(&mut self, ram: Capability<RamObj>, perm: rustyl4api::vspace::Permission) -> *mut u8 {
        use vspace_allocator::{VSpaceEntry, VSpaceManError};
        // TODO: support large page
        let layout = Layout::from_size_align(4096, 4096).unwrap();
        let vaddr = self.vspace_alloc.allocate(layout);
        let root_cap_slot = self.vspace_alloc.root_cap_slot();
        while let Err(e) = self.vspace_alloc.install_entry(VSpaceEntry::new_frame(ram.clone()), vaddr, 4) {
            match e {
                VSpaceManError::SlotOccupied{level} => {
                    panic!("slot occupied at level {} vaddr {:x}", level, vaddr);
                }
                VSpaceManError::SlotTypeError{level} => {
                    panic!("wrong slot type at level {} vaddr {:x}", level, vaddr);
                }
                VSpaceManError::PageTableMiss{level} => {
                    let vtable_cap = self.alloc_object::<VTableObj>(12).unwrap();
                    vtable_cap.map(root_cap_slot, vaddr, level + 1).unwrap();
                    let table_entry = VSpaceEntry::new_table(vtable_cap);
                    self.vspace_alloc.install_entry(table_entry, vaddr, level).unwrap();
                }
            }
        };
        ram.map(root_cap_slot, vaddr, perm).unwrap();
        vaddr as *mut u8
    }

    pub fn insert_vtable(&mut self, table: Capability<VTableObj>, vaddr: usize, level: usize) {
        let entry = vspace_allocator::VSpaceEntry::new_table(table);
        self.vspace_alloc
            .install_entry(entry, vaddr, level).unwrap();
    }

    pub fn install_ram(&mut self, ram: Capability<RamObj>, vaddr: usize) {
        let entry = vspace_allocator::VSpaceEntry::new_frame(ram);
        self.vspace_alloc
            .install_entry(entry, vaddr, 4).unwrap();
    }

    pub fn alloc_object<T: KernelObject>(&mut self, size: usize) -> Option<Capability<T>> {
        let slot = self.cspace_alloc()?;
        self.utspace_man.alloc_object::<T>(slot, size)
    }

    pub fn alloc_object_at<T: KernelObject>(&mut self, paddr: usize, bit_sz: usize, maybe_device: bool) -> Option<Capability<RamObj>> {
        unimplemented!();
    }
}