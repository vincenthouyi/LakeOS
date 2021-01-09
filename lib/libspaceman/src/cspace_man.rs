use core::sync::atomic::{AtomicUsize, Ordering};

use rustyl4api::object::CNodeCap;

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
    size: AtomicUsize,
    free_watermark: AtomicUsize,
}

impl CNodeBlock {
    pub const fn new(size: usize) -> Self {
        Self {
            size: AtomicUsize::new(size),
            free_watermark: AtomicUsize::new(0),
        }
    }

    pub fn alloc(&self) -> Option<usize> {
        loop {
            let cur_wm = self.free_watermark.load(Ordering::Relaxed);
            let node_sz = self.size.load(Ordering::Relaxed);
            if cur_wm < node_sz {
                let new_wm = cur_wm + 1;
                if cur_wm
                    == self
                        .free_watermark
                        .compare_and_swap(cur_wm, new_wm, Ordering::Relaxed)
                {
                    return Some(new_wm);
                }
            } else {
                break;
            }
        }
        None
    }

    pub fn alloc_at(&self, slot: usize) -> Option<usize> {
        loop {
            let cur_wm = self.free_watermark.load(Ordering::Relaxed);
            let node_sz = self.size.load(Ordering::Relaxed);
            if slot < node_sz && slot >= cur_wm {
                if cur_wm
                    == self
                        .free_watermark
                        .compare_and_swap(cur_wm, slot + 1, Ordering::Relaxed)
                {
                    return Some(slot);
                }
            } else {
                break;
            }
        }
        None
    }

    pub fn free(&self, _slot: usize) {}
}

#[derive(Debug)]
pub struct CSpaceMan {
    root_cnode: CNodeCap,
    root_cn_block: CNodeBlock,
    // root_cn_size: usize,
    // free_slots: LinkedList<SlotRange>
}

impl CSpaceMan {
    pub fn new(root_cnode: CNodeCap, root_cn_size: usize) -> Self {
        Self {
            root_cnode: root_cnode,
            root_cn_block: CNodeBlock::new(root_cn_size),
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

    //     let mut cur = self.free_slots.cursor_front_mut();
    //     let mut ret = None;

    //     loop {
    //         if let Some(node) = cur.current() {
    //             let range_start = node.start;
    //             let range_end = node.start + node.size;

    //             if slot < range_end && slot >= range_start {
    //                 ret = Some(slot);

    //                 if slot == range_end && slot == range_start {
    //                     cur.remove_current();
    //                 } else if slot == range_start {
    //                     node.start += 1;
    //                 } else if slot == range_end - 1 {
    //                     node.size -= 1;
    //                 } else {
    //                     node.size = slot - range_start;
    //                     let new_node = SlotRange::new(slot + 1, range_end - slot - 1);
    //                     cur.insert_after(new_node);
    //                     break;
    //                 }
    //             }

    //             cur.move_next();
    //         } else {
    //             break;
    //         }
    //     }

    //     ret
    // }

    // pub fn allocate_slot(&mut self) -> Option<usize> {
    //     let mut cur = self.free_slots.cursor_front_mut();
    //     let mut ret = None;
    //     if let Some(node) = cur.current() {
    //         ret = Some(node.start);
    //         node.start += 1;
    //         node.size -= 1;
    //         if node.size == 0 {
    //             cur.remove_current();
    //         }
    //     }
    //     ret
    // }
}
