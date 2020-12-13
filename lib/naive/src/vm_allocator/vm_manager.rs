use alloc::alloc::Layout;
use alloc::collections::LinkedList;
use crate::utils::align_up;

#[derive(Debug)]
pub enum VmType {
    Data,
    Slab,
    Malloc,
    Stack
}

#[derive(Debug)]
struct VmRange {
    start: usize,
    size: usize,
    // vm_type: VmType,
}

#[derive(Debug)]
pub struct VspaceAllocator {
    memlist: LinkedList<VmRange>,
    brk: usize,
}

impl VspaceAllocator {
    pub const fn new() -> Self {
        Self {
            memlist: LinkedList::new(),
            brk: 0,
        }
    }

    fn brk(&self) -> usize {
        self.memlist.iter().fold(0, |brk, e| brk.max(e.start + e.size))
    }

    pub fn allocate(&mut self, layout: Layout) -> usize {
        let start = align_up(self.brk(), layout.align());
        let size = layout.size();
        self.brk = start + size;
        self.memlist.push_back(VmRange{start: start, size: size});
        start
    }

    pub fn add_range(&mut self, start: usize, size: usize) {
        let range = VmRange{ start, size };
        self.memlist.push_back(range);
        self.brk = self.brk.max(start + size);
    }
}