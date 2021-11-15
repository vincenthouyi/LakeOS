use log::warn;

#[derive(Copy, Clone, Debug)]
pub struct RamBlock {
    pub base: usize,
    pub size: usize,
    pub cur: usize,
}

impl RamBlock {
    pub fn size(&self) -> usize {
        return self.size;
    }

    pub unsafe fn new(base: usize, size: usize) -> Self {
        return Self {
            base,
            size,
            cur: base,
        };
    }

    pub fn remain(&self) -> usize {
        return self.size - (self.cur - self.base);
    }

    pub fn top(&self) -> usize {
        self.base + self.size
    }

    pub fn frame_alloc(&mut self, size: usize) -> Option<*mut u8> {
        if self.remain() < size {
            return None;
        }
        let page = crate::boot::align_up(self.cur, size);
        self.cur = page + size;
        Some(page as *mut u8)
    }
}

#[derive(Debug)]
pub struct RamBlockList<const N: usize> {
    pub list: [Option<RamBlock>; N],
}

impl<const N: usize> RamBlockList<N> {
    pub fn new() -> Self {
        let entry = None;
        Self { list: [entry; N] }
    }

    pub fn insert(&mut self, base: usize, size: usize) {
        for i in 0..N {
            if self.list[i].is_none() {
                self.list[i] = unsafe { Some(RamBlock::new(base, size)) };
                return;
            }
        }
        warn!("Insert failed!")
    }

    pub fn frame_alloc(&mut self, size: usize) -> Option<*mut u8> {
        for maybe_blk in self.list.iter_mut() {
            if let Some(blk) = maybe_blk {
                let page = blk.frame_alloc(size);
                if page.is_some() {
                    return page;
                }
            }
        }
        return None;
    }
}
