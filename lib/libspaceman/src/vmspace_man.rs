use alloc::alloc::Layout;
use core::sync::atomic::{AtomicUsize, Ordering};

#[derive(Copy, Clone, Debug, Default)]
pub struct VmArea {
    start: usize,
    end: usize,
}

#[derive(Debug, Default)]
pub struct VMSpaceMan {
    // vma_list: LinkedList<VmArea>,
    // code_start: usize,
    // code_end: usize,
    // start_data: usize,
    // end_data: usize,
    // start_brk: usize,
    end_brk: AtomicUsize,
    // start_stack: usize,
    // start_arg: usize,
    // end_arg: usize,
    // start_env: usize,
    // end_env: usize,
}

impl VMSpaceMan {
    pub fn new() -> Self {
        Self {
            // vma_list : LinkedList::default(),
            end_brk: AtomicUsize::new(0x8000000000),
        }
    }

    // pub fn insert_vma(&mut self, start: usize, end: usize) {
    //     let vma = VmArea {start, end};
    //     let mut cur = self.vma_list.cursor_front_mut();

    //     loop {
    //         if cur.current().is_none() {
    //             break;
    //         }
    //         if cur.current().unwrap().start > start {
    //             break;
    //         }
    //         cur.move_next();
    //     }

    //     cur.insert_before(vma)
    // }

    // pub fn find_vma(&self, addr: usize) -> Option<&VmArea> {
    //     self.vma_list.iter().find(|vma| vma.end > addr)
    // }

    pub fn allocate_mem(&self, layout: Layout) -> usize {
        use crate::utils::align_up;

        loop {
            let start = align_up(self.end_brk.load(Ordering::Relaxed), layout.align());
            let size = layout.size();
            let end = start + size;
            if let Ok(start) = self.end_brk.compare_exchange(start, end, Ordering::Relaxed, Ordering::Relaxed) {
                return start;
            }
        }
        //        self.vma_list.push_back(VmArea{start, end});
    }
}
