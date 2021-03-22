use alloc::vec::Vec;
use crate::objects::{Capability, CapSlot, KernelObject, UntypedObj, UntypedCap};
use spin::Mutex;

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
    ut_list: Mutex<Vec<UntypedNode>>,
    // empty_ut: Vec<Vec<UntypedNode>>,
    // partial_ut: Vec<Vec<UntypedNode>>,
    // full_ut: Vec<Vec<UntypedNode>>,
}

impl UntypedSpaceMan {
    pub fn new() -> Self {
        Self {
            ut_list: Mutex::new(Vec::new()),
            // empty_ut: Vec::new(),
            // partial_ut: Vec::new(),
            // full_ut: Vec::new(),
        }
    }

    pub fn insert_untyped(
        &self,
        cap: UntypedCap,
        paddr: usize,
        bit_sz: u8,
        is_device: bool,
        free_offset: usize,
    ) {
        // TODO: support device untypeds
        if is_device {
            core::mem::forget(cap);
            return;
        }

        // TODO: support inserting non-empty untypeds
        if free_offset != 0 {
            core::mem::forget(cap);
            return;
        }

        if bit_sz < 4 {
            core::mem::forget(cap);
            return;
        }

        // let sz_offset = bit_sz - 4;

        // if self.empty_ut.len() <= sz_offset as usize {
        //     self.empty_ut.resize_with(sz_offset as usize + 1, || Vec::new());
        // }

        self.ut_list.lock().push(UntypedNode::new_empty(cap, paddr));
        // self.empty_ut[sz_offset as usize].push(UntypedNode::new_empty(cap, paddr));
    }

    pub fn alloc_object<T: KernelObject>(
        &self,
        dest_slot: CapSlot,
        size: usize,
    ) -> Option<Capability<T>> {
        for node in self.ut_list.lock().iter() {
            if let Ok(_) = node.cap.retype(T::obj_type(), size, dest_slot.slot(), 1) {
                return Some(Capability::<T>::new(dest_slot));
            }
        }

        None
        // use rustyl4api::init::InitCSpaceSlot::UntypedStart;

        // let untyped_cap = Capability::<UntypedObj>::new(UntypedStart as usize);
        // untyped_cap.retype(T::obj_type(), size, dest_slot, 1).ok()?;
        // Some(Capability::<T>::new(dest_slot))
    }
}
