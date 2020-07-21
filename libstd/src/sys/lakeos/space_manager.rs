use core::ptr::NonNull;

use rustyl4api::object::KernelObject;
use rustyl4api::object::{Capability, CNodeObj, RamObj, VTableObj};
use spaceman::SpaceManager;
use spin::Mutex;

pub static GLOBAL_SPACEMAN: Mutex<Option<SpaceManager>> = Mutex::new(None);

pub fn gsm_init(root_cnode: Capability<CNodeObj>, root_cnode_size: usize, root_vnode: Capability<VTableObj>) {
    *GLOBAL_SPACEMAN.lock() = Some(SpaceManager::new(root_cnode, root_cnode_size, root_vnode));
}

pub macro gsm {
    () => ({
        if super::vm_allocator::GLOBAL_VM_ALLOC.cur_pool_remain() < 512 {
            use rustyl4api::vspace::{FRAME_SIZE, Permission};

            let addr = GLOBAL_SPACEMAN.lock().as_mut().unwrap().map_frame_at(0, 0, FRAME_SIZE, Permission::writable()).unwrap();
            super::vm_allocator::GLOBAL_VM_ALLOC.add_mempool(addr, FRAME_SIZE);
        }
        GLOBAL_SPACEMAN.lock().as_mut().unwrap()
    }),
}

pub fn alloc_object_at<T: KernelObject>(paddr: usize, bit_sz: usize, maybe_device: bool) -> Option<Capability<RamObj>> {
    use rustyl4api::object::MonitorObj;
    use rustyl4api::init::InitCSpaceSlot::Monitor;

    let monitor_cap = Capability::<MonitorObj>::new(Monitor as usize);
    let ut_slot = gsm!().cspace_alloc()?;
    let ut_cap = monitor_cap.mint_untyped(ut_slot, paddr, bit_sz, maybe_device).ok()?;
    let obj_slot = gsm!().cspace_alloc()?;
    ut_cap.retype_one(bit_sz, obj_slot).ok()
}

pub fn allocate_frame_at(paddr: usize, _size: usize) -> Option<NonNull<u8>> {
    use rustyl4api::vspace::{Permission, FRAME_BIT_SIZE};

    let ram_obj = alloc_object_at::<RamObj>(paddr, FRAME_BIT_SIZE, true)
                    .unwrap();
    let vaddr = gsm!().insert_ram_at(ram_obj.clone(), 0, Permission::writable());

    NonNull::new(vaddr)
}