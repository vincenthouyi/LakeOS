use crate::utils::prev_power_of_two;
use core::alloc::{AllocError, Layout};
use core::cmp::max;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

use super::linked_list::LinkedList;

pub const MEMPOOL_MAX_BITSZ: usize = super::SLAB_ALLOC_BITSZ;
pub const MEMPOOL_MIN_BITSZ: usize = 3;
const MEMPOOL_ARRAY_SZ: usize = MEMPOOL_MAX_BITSZ - MEMPOOL_MIN_BITSZ + 1;

// TODO: use a big lock now to prevent deadlock. shold move to lockfree in the future
#[derive(Debug)]
struct SlabPool {
    pool: Mutex<[LinkedList; MEMPOOL_ARRAY_SZ]>,
    size: AtomicUsize,
}

impl SlabPool {
    pub const fn new() -> Self {
        Self {
            pool: Mutex::new([LinkedList::new(); MEMPOOL_ARRAY_SZ]),
            size: AtomicUsize::new(0),
        }
    }

    pub fn add_pool(&self, base: *mut u8, size: usize) {
        let mut cur_ptr = base as usize;
        let mut rem_sz = size;

        while rem_sz > 0 {
            let cur_sz = (cur_ptr & (!cur_ptr + 1))
                .min(prev_power_of_two(rem_sz))
                .min(1 << MEMPOOL_MAX_BITSZ);
            let cur_bitsz = cur_sz.trailing_zeros() as usize;
            //            crate::println!("adding mempool {:p}-{:p} size {}", cur_ptr as *mut usize, (cur_ptr + cur_sz) as *mut usize, cur_sz);

            if cur_bitsz >= MEMPOOL_MIN_BITSZ {
                unsafe {
                    self.pool.lock()[cur_bitsz - MEMPOOL_MIN_BITSZ].push(cur_ptr as *mut usize);
                }
                self.size.fetch_add(1 << cur_bitsz, Ordering::Relaxed);
            }
            cur_ptr += cur_sz;
            rem_sz -= cur_sz;
        }
    }

    pub fn slab_alloc(&self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        let bit_sz = chunk_size(layout).trailing_zeros() as usize;

        (bit_sz..=MEMPOOL_MAX_BITSZ)
            .find_map(|sz| {
                self.pool.lock()[sz - MEMPOOL_MIN_BITSZ]
                    .pop()
                    .map(|ptr| (sz, ptr as *mut u8))
            })
            .map(|(chunk_sz, ptr)| unsafe {
                //                crate::println!("getting ptr {:p} size {}", ptr, 1 << chunk_sz);
                for sz in bit_sz..chunk_sz {
                    let back_sz = 1 << sz;
                    let back_ptr = ptr.offset(back_sz);
                    //                    crate::println!("inserting back {:p} size {}", back_ptr, back_sz);
                    self.pool.lock()[sz - MEMPOOL_MIN_BITSZ].push(back_ptr as *mut usize)
                }
                self.size.fetch_sub(1 << bit_sz, Ordering::Relaxed);

                NonNull::new_unchecked(ptr as *mut u8)
            })
            .ok_or(AllocError {})
    }

    pub fn slab_dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        let mut bit_sz = chunk_size(layout).trailing_zeros() as usize;
        let mut cur_ptr = ptr.as_ptr() as usize;

        if layout.size() > 4096 {
            // TODO: dealloc mem > 4096
            return;
        }

        self.size.fetch_add(chunk_size(layout), Ordering::Relaxed);

        while bit_sz < MEMPOOL_MAX_BITSZ {
            let buddy = (cur_ptr ^ (1 << bit_sz)) as *mut usize;
            let tmp_ptr = self.pool.lock()[bit_sz - MEMPOOL_MIN_BITSZ]
                .iter_mut()
                .find(|node| node.value() == buddy)
                .map(|node| node.pop() as usize);

            if tmp_ptr.is_none() {
                break;
            } else {
                cur_ptr = cur_ptr & !(1 << bit_sz);
                bit_sz += 1;
            }
        }

        unsafe {
            self.pool.lock()[bit_sz - MEMPOOL_MIN_BITSZ].push(cur_ptr as *mut usize);
        }
    }

    pub fn size(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }
}

#[derive(Debug)]
pub struct SlabAllocator {
    mempool: [SlabPool; 2],
    cur_pool: AtomicUsize,
}

impl SlabAllocator {
    pub const fn new() -> Self {
        Self {
            mempool: [SlabPool::new(), SlabPool::new()],
            cur_pool: AtomicUsize::new(0),
        }
    }

    fn current_pool(&self) -> &SlabPool {
        &self.mempool[self.cur_pool.load(Ordering::Relaxed)]
    }

    fn backup_pool(&self) -> &SlabPool {
        &self.mempool[self.cur_pool.load(Ordering::Relaxed) ^ 1]
    }

    pub fn swap_pool(&self) {
        self.cur_pool.fetch_xor(1, Ordering::Relaxed);
    }

    pub fn add_mempool(&self, base: *mut u8, size: usize) {
        self.current_pool().add_pool(base, size)
    }

    pub fn add_backup_mempool(&self, base: *mut u8, size: usize) {
        self.backup_pool().add_pool(base, size)
    }

    pub fn slab_alloc(&self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        self.current_pool().slab_alloc(layout)
    }

    pub fn slab_dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        self.current_pool().slab_dealloc(ptr, layout)
    }

    pub fn cur_pool_remain(&self) -> usize {
        self.current_pool().size()
    }
}

fn chunk_size(layout: Layout) -> usize {
    use core::mem::size_of;

    const SIZEOF_USIZE: usize = size_of::<usize>();

    max(
        layout.size().next_power_of_two(),
        max(layout.align(), SIZEOF_USIZE),
    )
}
