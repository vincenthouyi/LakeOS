use alloc::vec::Vec;
use rustyl4api::object::{Capability, KernelObject, UntypedObj};

#[derive(Debug)]
struct UntypedNode {
    paddr: usize,
    cap: Capability<UntypedObj>,
    free_offset: usize,
}

impl UntypedNode {
    pub const fn new_empty(cap: Capability<UntypedObj>, paddr: usize) -> Self {
        Self {
            paddr: paddr,
            cap: cap,
            free_offset: 0,
        }
    }
}

#[derive(Debug)]
pub struct UntypedSpaceMan {
    empty_ut: Vec<Vec<UntypedNode>>,
    // partial_ut: Vec<Vec<UntypedNode>>,
    // full_ut: Vec<Vec<UntypedNode>>,
}

impl UntypedSpaceMan {
    pub fn new() -> Self {
        Self {
            empty_ut: Vec::new(),
            // partial_ut: Vec::new(),
            // full_ut: Vec::new(),
        }
    }

    pub fn insert_untyped(&mut self, slot: usize, paddr: usize, bit_sz: u8, is_device: bool, free_offset: usize) {
        // TODO: support device untypeds
        if is_device {
            return;
        }

        // TODO: support inserting non-empty untypeds
        if free_offset != 0 {
            return;
        }

        if bit_sz < 4 {
            return;
        }

        let sz_offset = bit_sz - 4;

        if self.empty_ut.len() <= sz_offset as usize {
            self.empty_ut.resize_with(sz_offset as usize + 1, || Vec::new());
        }

        let cap = Capability::new(slot);
        self.empty_ut[sz_offset as usize].push(UntypedNode::new_empty(cap, paddr));
    }

    pub fn alloc_object<T: KernelObject>(&mut self, dest_slot: usize, size: usize) -> Option<Capability<T>> {
        use rustyl4api::init::InitCSpaceSlot::UntypedStart;

        let untyped_cap = Capability::<UntypedObj>::new(UntypedStart as usize);
        untyped_cap.retype(T::obj_type(), size, dest_slot, 1).ok()?;
        Some(Capability::<T>::new(dest_slot))
    }
}