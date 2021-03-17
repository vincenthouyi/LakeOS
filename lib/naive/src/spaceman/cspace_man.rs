use core::ops::Range;

use alloc::collections::LinkedList;

use spin::Mutex;

use crate::objects::CNodeCap;

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
    cap: CNodeCap,
    range: Range<usize>,
    free_slots: Mutex<LinkedList<Range<usize>>>,
}

impl CNodeBlock {
    pub fn new(cap: CNodeCap, start: usize, size: usize) -> Self {
        let mut free_slots = LinkedList::new();
        free_slots.push_back(start .. start + size);
        Self {
            cap: cap,
            range: start .. start + size,
            free_slots: Mutex::new(free_slots)
        }
    }

    pub fn alloc(&self) -> Option<usize> {
        let mut free_slots_guard = self.free_slots.lock();
        let mut range = free_slots_guard.front_mut()?;

        let ret_slot = range.start;
        range.start += 1;
        let front_empty = range.is_empty();

        drop(range);

        if front_empty {
            free_slots_guard.pop_front();
        }

        Some(ret_slot)
    }

    pub fn alloc_at(&self, slot: usize) -> Option<usize> {
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
                        cur.insert_before(cur_range.start .. slot);
                        cur.insert_after(slot + 1 .. cur_range.end);
                        cur.remove_current().unwrap();
                    }
                    break;
                }
                cur.move_next();
            } else {
                return None;
            }
        }

        ret_slot
    }

    pub fn free(&self, slot: usize) {
        let mut free_slots_guard = self.free_slots.lock();
        let mut cur = free_slots_guard.cursor_front_mut();

        loop {
            if let Some(range) = cur.current() {
                if slot == range.start - 1 {
                    range.start -= 1;
                    return;
                } else if slot == range.end {
                    range.end += 1;
                    let range = range.clone();
                    if let Some(next_range) = cur.peek_next() {
                        if range.end == next_range.start{
                            next_range.start = range.start;
                            cur.move_prev();
                            cur.remove_current();
                            // range.end = next_range.end;
                            // cur.move_next();
                            // cur.remove_current();
                        }
                    }
                    return;
                }
            } else {
                cur.insert_after(slot..slot+1);
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
    pub fn new(root_cnode: CNodeCap, root_cn_size: usize) -> Self {
        Self {
            root_cn_block: CNodeBlock::new(root_cnode, 0, root_cn_size),
        }
    }

    pub fn allocate_slot_at(&self, slot: usize) -> Option<usize> {
        self.root_cn_block.alloc_at(slot)
    }

    pub fn allocate_slot(&self) -> Option<usize> {
        self.root_cn_block.alloc()
    }

    pub fn free_slot(&self, slot: usize) {
        self.root_cn_block.free(slot)
    }
}
