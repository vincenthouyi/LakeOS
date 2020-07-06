use core::ptr::NonNull;

use rustyl4api::object::KernelObject;
use rustyl4api::object::{Capability, CNodeObj, RamObj, VTableObj};
use spaceman::SpaceManager;
use mutex::Mutex;

pub struct GlobalAllocator(Mutex<Option<SpaceManager>>);

impl GlobalAllocator {
    pub const fn uninitialized() -> Self {
        Self(Mutex::new(None))
    }

    pub fn initialize(&self, root_cnode: Capability<CNodeObj>, root_cnode_size: usize, root_vnode: Capability<VTableObj>, brk: usize) {
        *self.0.lock() = Some(SpaceManager::new(root_cnode, root_cnode_size, root_vnode, brk));
    }

    pub fn new(root_cnode: Capability<CNodeObj>, root_cnode_size: usize, root_vnode: Capability<VTableObj>, brk: usize) -> Self {
        let x = Self::uninitialized();
        x.initialize(root_cnode, root_cnode_size, root_vnode, brk);
        x
    }

    pub fn insert_untyped(&self, slot: usize, paddr: usize, bit_sz: u8, is_device: bool, free_offset: usize) {
        self.0
            .lock()
            .as_mut().unwrap()
            .insert_untyped(slot, paddr, bit_sz, is_device, free_offset)
    }

    pub fn cspace_alloc(&self) -> Option<usize> {
        self.0
            .lock()
            .as_mut()?
            .cspace_alloc()
    }

    pub fn cspace_alloc_at(&self, slot: usize) -> Option<usize> {
        self.0
            .lock()
            .as_mut()?
            .cspace_alloc_at(slot)
    }

    pub fn alloc_object<T: KernelObject>(&self, size: usize) -> Option<Capability<T>> {
        self.0
            .lock()
            .as_mut()?
            .alloc_object::<T>(size)
    }

    pub fn insert_ram(&self, ram: Capability<RamObj>, perm: rustyl4api::vspace::Permission) -> *mut u8 {
        self.0
            .lock()
            .as_mut()
            .expect("allocator not initialized")
            .insert_ram(ram, perm)
    }

    pub fn alloc_object_at<T: KernelObject>(&self, paddr: usize, bit_sz: usize, maybe_device: bool) -> Option<Capability<RamObj>> {
        use rustyl4api::object::MonitorObj;
        use rustyl4api::init::InitCSpaceSlot::Monitor;

        let monitor_cap = Capability::<MonitorObj>::new(Monitor as usize);
        let ut_slot = self.cspace_alloc()?;
        let ut_cap = monitor_cap.mint_untyped(ut_slot, paddr, bit_sz, maybe_device).ok()?;
        let obj_slot = self.cspace_alloc()?;
        ut_cap.retype_one(bit_sz, obj_slot).ok()
    }

    pub fn insert_vtable(&self, table: Capability<VTableObj>, vaddr: usize, level: usize) {
        self.0
            .lock()
            .as_mut().unwrap()
            .insert_vtable(table, vaddr, level)
    }

    pub fn install_ram(&self, ram: Capability<RamObj>, vaddr: usize) {
        self.0
            .lock()
            .as_mut().unwrap()
            .install_ram(ram, vaddr)
    }
}

pub static INIT_ALLOC: GlobalAllocator = GlobalAllocator::uninitialized();

pub fn allocate_frame_at(paddr: usize, _size: usize) -> Option<NonNull<u8>> {
    use rustyl4api::vspace::{Permission, FRAME_BIT_SIZE};

    let ram_obj = INIT_ALLOC.alloc_object_at::<RamObj>(paddr, FRAME_BIT_SIZE, true)
                    .unwrap();
    let vaddr = INIT_ALLOC.insert_ram(ram_obj.clone(), Permission::writable());

    NonNull::new(vaddr)
}