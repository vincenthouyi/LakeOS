use core::ops::Range;

use alloc::collections::LinkedList;

use spin::Mutex;

use crate::objects::{CNodeRef, CapSlot};
use rustyl4api::process::ProcessCSpace;

// #[derive(Debug)]
// struct SlotRange {
//     start: usize,
//     size: usize,
// }

// impl SlotRange {
//     pub fn new(start: usize, size: usize) -> Self {
//         Self {start: start, size: size}
//     }
// }

#[derive(Debug)]
struct CNodeBlock {
    cap: CNodeRef,
    range: Range<usize>,
    free_slots: Mutex<LinkedList<Range<usize>>>,
}

impl CNodeBlock {
    pub fn new(cap: CNodeRef, start: usize, size: usize) -> Self {
        let mut free_slots = LinkedList::new();
        free_slots.push_back(start..start + size);
        Self {
            cap: cap,
            range: start..start + size,
            free_slots: Mutex::new(free_slots),
        }
    }

    pub fn alloc(&self) -> Option<CapSlot> {
        let mut free_slots_guard = self.free_slots.lock();
        let mut range = free_slots_guard.front_mut()?;

        let ret_slot = range.start;
        range.start += 1;
        let front_empty = range.is_empty();

        drop(range);

        if front_empty {
            free_slots_guard.pop_front();
        }

        Some(CapSlot::new(ret_slot))
    }

    pub fn alloc_at(&self, slot: usize) -> Option<CapSlot> {
        let mut free_slots_guard = self.free_slots.lock();
        let mut cur = free_slots_guard.cursor_front_mut();
        let mut ret_slot = None;

        while let None = ret_slot {
            if let Some(range) = cur.current() {
                if range.contains(&slot) {
                    ret_slot = Some(slot);
                    if range.start == slot {
                        range.start += 1;
                        if range.is_empty() {
                            cur.remove_current();
                        }
                    } else if range.end == slot {
                        range.end -= 1;
                        if range.is_empty() {
                            cur.remove_current();
                        }
                    } else {
                        let cur_range = range.clone();
                        cur.insert_before(cur_range.start..slot);
                        cur.insert_after(slot + 1..cur_range.end);
                        cur.remove_current().unwrap();
                    }
                    break;
                }
                cur.move_next();
            } else {
                return None;
            }
        }

        ret_slot.map(|s| CapSlot::new(s))
    }

    pub fn free(&self, slot: usize) {
        let mut free_slots_guard = self.free_slots.lock();
        let mut cur = free_slots_guard.cursor_front_mut();

        if slot < ProcessCSpace::WellKnownMax as usize {
            kprintln!("Warning: trying to free well known slot!");
            return;
        }

        if !self.range.contains(&slot) {
            kprintln!(
                "Warning: freeing slot {} not in block range {:?}",
                slot,
                self.range
            );
            return;
        }

        loop {
            if let Some(range) = cur.current() {
                if slot == range.start - 1 {
                    range.start -= 1;
                    return;
                } else if slot == range.end {
                    range.end += 1;
                    let range = range.clone();
                    if let Some(next_range) = cur.peek_next() {
                        if range.end == next_range.start {
                            next_range.start = range.start;
                            cur.remove_current();
                        }
                    }
                    return;
                } else if range.contains(&slot) {
                    kprintln!("Warning: cap slot {} is double freed!", slot);
                    return;
                }
                cur.move_next();
            } else {
                cur.insert_after(slot..slot + 1);
                return;
            }
        }
    }
}

#[derive(Debug)]
pub struct CSpaceMan {
    root_cn_block: CNodeBlock,
}

impl CSpaceMan {
    pub fn new(root_cnode: CNodeRef, root_cn_size: usize) -> Self {
        Self {
            root_cn_block: CNodeBlock::new(
                root_cnode,
                ProcessCSpace::WellKnownMax as usize,
                root_cn_size - ProcessCSpace::WellKnownMax as usize,
            ),
        }
    }

    pub fn allocate_slot_at(&self, slot: usize) -> Option<CapSlot> {
        self.root_cn_block.alloc_at(slot)
    }

    pub fn allocate_slot(&self) -> Option<CapSlot> {
        self.root_cn_block.alloc()
    }

    pub fn free_slot(&self, slot: usize) {
        self.root_cn_block.free(slot)
    }

    pub fn root_cnode(&self) -> CNodeRef {
        self.root_cn_block.cap.clone()
    }
}
