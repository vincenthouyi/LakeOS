use core::ptr::NonNull;

use rustyl4api::object::KernelObject;
use rustyl4api::object::{CNodeObj, Capability, RamObj, VTableObj};
use crate::spaceman::SpaceManager;

pub static mut GLOBAL_SPACEMAN: Option<SpaceManager> = None;

pub fn gsm_init(
    root_cnode: Capability<CNodeObj>,
    root_cnode_size: usize,
    root_vnode: Capability<VTableObj>,
) {
    unsafe {
        GLOBAL_SPACEMAN = Some(SpaceManager::new(root_cnode, root_cnode_size, root_vnode));
    }
}

pub macro gsm() {
    unsafe {
        if crate::vm_allocator::GLOBAL_VM_ALLOC.cur_pool_remain() < 1024 {
            use rustyl4api::vspace::{Permission, FRAME_SIZE};

            let addr = GLOBAL_SPACEMAN
                .as_mut()
                .unwrap()
                .map_frame_at(0, 0, FRAME_SIZE, Permission::writable())
                .unwrap();
            crate::vm_allocator::GLOBAL_VM_ALLOC.add_mempool(addr, FRAME_SIZE);
        }
        GLOBAL_SPACEMAN.as_mut().unwrap()
    }
}

pub fn alloc_object_at<T: KernelObject>(
    paddr: usize,
    bit_sz: usize,
    maybe_device: bool,
) -> Option<Capability<RamObj>> {
    use rustyl4api::init::InitCSpaceSlot::Monitor;
    use rustyl4api::object::MonitorObj;

    let monitor_cap = Capability::<MonitorObj>::new(Monitor as usize);
    let ut_slot = gsm!().cspace_alloc()?;
    let ut_cap = monitor_cap
        .mint_untyped(ut_slot, paddr, bit_sz, maybe_device)
        .ok()?;
    let obj_slot = gsm!().cspace_alloc()?;
    ut_cap.retype_one(bit_sz, obj_slot).ok()
}

pub fn allocate_frame_at(paddr: usize, _size: usize) -> Option<NonNull<u8>> {
    use rustyl4api::vspace::{Permission, FRAME_BIT_SIZE};

    let ram_obj = alloc_object_at::<RamObj>(paddr, FRAME_BIT_SIZE, true).unwrap();
    let vaddr = gsm!().insert_ram_at(ram_obj.clone(), 0, Permission::writable());

    NonNull::new(vaddr)
}
