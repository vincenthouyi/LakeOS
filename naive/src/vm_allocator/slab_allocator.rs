use mutex::Mutex;
use core::alloc::{Layout, AllocErr};
use core::ptr::{NonNull};
use core::cmp::max;
use crate::utils::prev_power_of_two;

use super::linked_list::LinkedList;

pub const MEMPOOL_MAX_BITSZ: usize = super::SLAB_ALLOC_BITSZ;
pub const MEMPOOL_MIN_BITSZ: usize = 3;
const MEMPOOL_ARRAY_SZ: usize = MEMPOOL_MAX_BITSZ - MEMPOOL_MIN_BITSZ + 1;

#[derive(Debug)]
pub struct SlabAllocator {
    mempool: [[Mutex<LinkedList>; MEMPOOL_ARRAY_SZ]; 2],
    cur_pool: Mutex<usize>,
}

impl SlabAllocator {
    pub const fn new() -> Self {
        Self {
            mempool: [[Mutex::new(LinkedList::new()); MEMPOOL_ARRAY_SZ],
                      [Mutex::new(LinkedList::new()); MEMPOOL_ARRAY_SZ]],
            cur_pool: Mutex::new(0),
        }
    }

    fn current_pool(&self) -> &[Mutex<LinkedList>; MEMPOOL_ARRAY_SZ] {
        &self.mempool[*self.cur_pool.lock()]
    }

    fn backup_pool(&self) -> &[Mutex<LinkedList>; MEMPOOL_ARRAY_SZ] {
        &self.mempool[*self.cur_pool.lock() ^ 1]
    }

    pub fn swap_pool(&self) {
        let mut guard = self.cur_pool.lock();
        let p = *guard ^ 1;
        *guard = p;
    }

    fn _add_mempool(&self, base: *mut u8, size: usize, backup: bool) {
        let mut cur_ptr = base as usize;
        let mut rem_sz = size;
        let pool = if backup {
            self.backup_pool()
        } else {
            self.current_pool()
        };
//        crate::println!("mempool total {:p}-{:p} size {}", base, (base as usize + size) as *mut u8, size);

        while rem_sz > 0 {
            let cur_sz = (cur_ptr & (!cur_ptr + 1))
                .min(prev_power_of_two(rem_sz))
                .min(1 << MEMPOOL_MAX_BITSZ);
            let cur_bitsz = cur_sz.trailing_zeros() as usize;
//            crate::println!("adding mempool {:p}-{:p} size {}", cur_ptr as *mut usize, (cur_ptr + cur_sz) as *mut usize, cur_sz);

            if cur_bitsz >= MEMPOOL_MIN_BITSZ {
                unsafe {
                    pool[cur_bitsz - MEMPOOL_MIN_BITSZ]
                        .lock()
                        .push(cur_ptr as *mut usize);
                }
            }
            cur_ptr += cur_sz;
            rem_sz -= cur_sz;
        }
    }

    pub fn add_mempool(&self, base: *mut u8, size: usize) {
        self._add_mempool(base, size, false)
    }

    pub fn add_backup_mempool(&self, base: *mut u8, size: usize) {
        self._add_mempool(base, size, true)
    }

    pub fn slab_alloc(&self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        let bit_sz = chunk_size(layout).trailing_zeros() as usize;

        (bit_sz..=MEMPOOL_MAX_BITSZ)
            .find_map(|sz|
                self.current_pool()[sz - MEMPOOL_MIN_BITSZ]
                    .lock()
                    .pop()
                    .map(|ptr| (sz, ptr as *mut u8))
            )
            .map(|(chunk_sz, ptr)| unsafe {
//                crate::println!("getting ptr {:p} size {}", ptr, 1 << chunk_sz);
                for sz in bit_sz..chunk_sz {
                    let back_sz = 1 << sz;
                    let back_ptr = ptr.offset(back_sz);
//                    crate::println!("inserting back {:p} size {}", back_ptr, back_sz);
                    self.current_pool()[sz - MEMPOOL_MIN_BITSZ]
                        .lock()
                        .push(back_ptr as *mut usize)
                }

                NonNull::new_unchecked(ptr as *mut u8)
            })
            .ok_or(AllocErr {})
    }

    pub fn slab_dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        let mut bit_sz = chunk_size(layout).trailing_zeros() as usize;
        let mut cur_ptr = ptr.as_ptr() as usize;

        while bit_sz < MEMPOOL_MAX_BITSZ {
            let buddy = (cur_ptr ^ (1 << bit_sz)) as *mut usize;
            let tmp_ptr = self.current_pool()[bit_sz - MEMPOOL_MIN_BITSZ]
                              .lock()
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
            self.current_pool()[bit_sz - MEMPOOL_MIN_BITSZ]
                .lock()
                .push(cur_ptr as *mut usize);
        }
    }
}

fn chunk_size(layout: Layout) -> usize {
    use core::mem::size_of;

    const SIZEOF_USIZE: usize = size_of::<usize>();

    max(layout.size().next_power_of_two(),
        max(layout.align(), SIZEOF_USIZE))
}
