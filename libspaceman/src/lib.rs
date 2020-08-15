#![feature(linked_list_cursors)]
#![feature(const_saturating_int_methods)]

#![no_std]

extern crate alloc;

use rustyl4api::object::KernelObject;
use rustyl4api::vspace::Permission;

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
pub mod vspace_man;
pub mod cspace_man;
pub mod utspace_man;
pub mod vmspace_man;

use core::alloc::Layout;

use rustyl4api::object::{Capability, CNodeObj, RamObj, VTableObj};
use rustyl4api::object::identify::IdentifyResult;

#[derive(Debug)]
pub struct SpaceManager {
    vspace_man: vspace_man::VSpaceMan,
    cspace_man: cspace_man::CSpaceMan,
    utspace_man: utspace_man::UntypedSpaceMan,
    vmspace_man: vmspace_man::VMSpaceMan,
}

impl SpaceManager {
    pub fn new(root_cnode: Capability<CNodeObj>, root_cnode_size: usize, root_vnode: Capability<VTableObj>) -> Self {
        Self {
            vspace_man: vspace_man::VSpaceMan::new(root_vnode),
            cspace_man: cspace_man::CSpaceMan::new(root_cnode, root_cnode_size),
            utspace_man: utspace_man::UntypedSpaceMan::new(),
            vmspace_man: vmspace_man::VMSpaceMan::new(),
        }
    }

    pub fn insert_untyped(&self, slot: usize, paddr: usize, bit_sz: u8, is_device: bool, free_offset: usize) {
        self.utspace_man
            .insert_untyped(slot, paddr, bit_sz, is_device, free_offset)
    }

    // pub fn insert_vm_range(&mut self, start: usize, end: usize) {
    //     self.vmspace_man.insert_vma(start, end);
    // }

    pub fn insert_identify(&self, slot: usize, result: IdentifyResult) {
        self.cspace_alloc_at(slot);
        match result {
            IdentifyResult::NullObj => { },
            IdentifyResult::Untyped {paddr, bit_sz, is_device, free_offset} => {
                // self.utspace_man.insert_untyped(slot, paddr, bit_sz, is_device, free_offset)
                self.insert_untyped(slot, paddr, bit_sz, is_device, free_offset);
            },
            IdentifyResult::CNode {bit_sz} => { },
            IdentifyResult::Tcb => { },
            IdentifyResult::Ram {bit_sz, mapped_vaddr, mapped_asid, is_device} => {
                let cap = Capability::<RamObj>::new(slot);
                self.install_ram(cap, mapped_vaddr);
            },
            IdentifyResult::VTable {mapped_vaddr, mapped_asid, level} => {
                let table = Capability::<VTableObj>::new(slot);
                self.insert_vtable(table, mapped_vaddr, level - 1);
            },
            IdentifyResult::Endpoint => { },
            IdentifyResult::Reply => { },
            IdentifyResult::Monitor => { },
            IdentifyResult::Interrupt => { },
        }
    }

    pub fn cspace_alloc_at(&self, slot: usize) -> Option<usize> {
        self.cspace_man
            .allocate_slot_at(slot)
    }

    pub fn cspace_alloc(&self) -> Option<usize> {
        self.cspace_man
            .allocate_slot()
    }

    pub fn cspace_free(&self, slot: usize) {
        unimplemented!()
    }

    pub fn vspace_alloc(&self, layout: Layout) -> Option<usize> {
        Some(self.vmspace_man.allocate_mem(layout))
    }

    pub fn map_frame_at(&self, paddr: usize, vaddr: usize, size: usize, perm: Permission) -> Result<*mut u8, ()> {
        if paddr != 0 {
            return Err(());
        }

        let size = utils::align_up(size, 4096);
        if size > 4096 {
            return Err(());
        }

        let bit_sz = size.trailing_zeros() as usize;

        let frame = self.alloc_object::<RamObj>(bit_sz).ok_or(())?;
        let vaddr = self.insert_ram_at(frame, vaddr, perm);

        Ok(vaddr)
    }

    /// Insert an RamCap to vspace to manage and handle backed page table
    pub fn insert_ram_at(&self, ram: Capability<RamObj>, vaddr: usize, perm: Permission) -> *mut u8 {
        use rustyl4api::error::SysError;
        // TODO: support large page
        let layout = Layout::from_size_align(4096, 4096).unwrap();
        let vaddr = if vaddr == 0 {
            self.vspace_alloc(layout).unwrap()
        } else {
            vaddr
        };
        while let Err(e) = self.vspace_man.map_frame(ram.clone(), vaddr, perm, 4) {
            match e {
                // VSpaceManError::SlotOccupied{level} => {
                //     panic!("slot occupied at level {} vaddr {:x}", level, vaddr);
                // }
                // VSpaceManError::SlotTypeError{level} => {
                //     panic!("wrong slot type at level {} vaddr {:x}", level, vaddr);
                // }
                SysError::VSpaceTableMiss{level} => {
                    let vtable_cap = self.alloc_object::<VTableObj>(12).unwrap();
                    self.vspace_man.map_table(vtable_cap, vaddr, level as usize).unwrap();
                }
                e => {
                    panic!("unexpected error {:?}", e);
                }
            }
        };
        vaddr as *mut u8
    }

    pub fn insert_vtable(&self, table: Capability<VTableObj>, vaddr: usize, level: usize) {
        let entry = vspace_man::VSpaceEntry::new_table(table);
        self.vspace_man
            .install_entry(entry, vaddr, level).unwrap();
    }

    pub fn install_ram(&self, ram: Capability<RamObj>, vaddr: usize) {
        let entry = vspace_man::VSpaceEntry::new_frame(ram);
        self.vspace_man
            .install_entry(entry, vaddr, 4).unwrap();
    }

    pub fn alloc_object<T: KernelObject>(&self, size: usize) -> Option<Capability<T>> {
        let slot = self.cspace_alloc()?;
        self.utspace_man.alloc_object::<T>(slot, size)
    }

    pub fn alloc_object_at<T: KernelObject>(&self, paddr: usize, bit_sz: usize, maybe_device: bool) -> Option<Capability<RamObj>> {
        unimplemented!();
    }
}