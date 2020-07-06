use alloc::collections::linked_list::LinkedList;

use rustyl4api::object::CNodeCap;

#[derive(Debug)]
struct SlotRange {
    start: usize,
    size: usize,
}

impl SlotRange {
    pub fn new(start: usize, size: usize) -> Self {
        Self {start: start, size: size}
    }
}

#[derive(Debug)]
pub struct CSpaceMan {
    root_cnode: CNodeCap,
    root_cn_size: usize,
    free_slots: LinkedList<SlotRange>
}

impl CSpaceMan {
    pub fn new(root_cnode: CNodeCap, root_cn_size: usize) -> Self {
        let mut free_slots = LinkedList::new();
        free_slots.push_back(SlotRange::new(0, root_cn_size));
        Self {
            root_cnode: root_cnode,
            root_cn_size: root_cn_size,
            free_slots: free_slots,
        }
    }

    pub fn allocate_slot_at(&mut self, slot: usize) -> Option<usize> {

        let mut cur = self.free_slots.cursor_front_mut();
        let mut ret = None;

        loop {
            if let Some(node) = cur.current() {
                let range_start = node.start;
                let range_end = node.start + node.size;

                if slot < range_end && slot >= range_start {
                    ret = Some(slot);

                    if slot == range_end && slot == range_start {
                        cur.remove_current();
                    } else if slot == range_start {
                        node.start += 1;
                    } else if slot == range_end - 1 {
                        node.size -= 1;
                    } else {
                        node.size = slot - range_start;
                        let new_node = SlotRange::new(slot + 1, range_end - slot - 1);
                        cur.insert_after(new_node);
                        break;
                    }
                }

                cur.move_next();
            } else {
                break;
            }
        }

        ret
    }

    pub fn allocate_slot(&mut self) -> Option<usize> {
        let mut cur = self.free_slots.cursor_front_mut();
        let mut ret = None;
        if let Some(node) = cur.current() {
            ret = Some(node.start);
            node.start += 1;
            node.size -= 1;
            if node.size == 0 {
                cur.remove_current();
            }
        }
        ret
    }
}