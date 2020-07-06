mod linked_list;
mod slab_allocator;

use core::alloc::{GlobalAlloc, Layout, AllocErr};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, Ordering};

use slab_allocator::SlabAllocator;

pub const SLAB_ALLOC_BITSZ: usize = rustyl4api::vspace::FRAME_BIT_SIZE;

#[derive(Debug)]
pub struct VmAllocator {
    slab_alloc: SlabAllocator,
    backup_empty: AtomicBool,
}

impl VmAllocator {
    pub const fn new() -> Self {
        VmAllocator {
            slab_alloc: SlabAllocator::new(),
            backup_empty: AtomicBool::new(false),
        }
    }

    pub fn add_mempool(&self, base: *mut u8, size: usize) {
        self.slab_alloc.add_mempool(base, size)
    }

    pub fn add_backup_mempool(&self, base: *mut u8, size: usize) {
        self.slab_alloc.add_backup_mempool(base, size)
    }

    pub fn slab_refill(&self, layout: Layout) {
        use rustyl4api::vspace::{FRAME_SIZE, Permission};
        use crate::space_manager::gsm;

        let addr = gsm!().map_frame_at(0, 0, FRAME_SIZE, Permission::writable()).unwrap();
        self.add_backup_mempool(addr, FRAME_SIZE);
    }

    pub fn vm_alloc(&self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {

        // TODO: support object larger than a page
        let obj_bitsz = layout.size().trailing_zeros();
        if obj_bitsz > SLAB_ALLOC_BITSZ as u32 {
            return Err(AllocErr{});
        }

        self.slab_alloc
            .slab_alloc(layout)
            .or_else(|_| {
                self.slab_alloc.swap_pool();
                self.slab_refill(layout);
                self.slab_alloc.slab_alloc(layout)
            })
    }

    pub fn vm_dealloc(&self, ptr: NonNull<u8>, layout: Layout) {
        self.slab_alloc
            .slab_dealloc(ptr, layout)
    }

    pub fn backup_empty(&self) -> bool {
        self.backup_empty.load(Ordering::Relaxed)
    }

    pub fn set_backup_empty(&self, b: bool) {
        self.backup_empty.store(b, Ordering::Relaxed);
    }

    pub fn cur_pool_remain(&self) -> usize {
        self.slab_alloc.cur_pool_remain()
    }
}

unsafe impl GlobalAlloc for VmAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        use core::ptr::null_mut;
        self.vm_alloc(layout).map(|p| p.as_ptr()).unwrap_or(null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.vm_dealloc(NonNull::new_unchecked(ptr), layout)
    }
}

#[global_allocator]
pub static GLOBAL_VM_ALLOC: VmAllocator = VmAllocator::new();

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}