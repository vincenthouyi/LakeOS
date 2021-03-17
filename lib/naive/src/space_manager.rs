use core::ptr::NonNull;

use crate::objects::KernelObject;
use crate::objects::{CNodeCap, Capability, RamObj, VTableCap};
use crate::spaceman::SpaceManager;

pub static mut GLOBAL_SPACEMAN: Option<SpaceManager> = None;

pub fn gsm_init(
    root_cnode: CNodeCap,
    root_cnode_size: usize,
    root_vnode: VTableCap,
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
    use crate::objects::MonitorObj;

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
    let vaddr = gsm!().insert_ram_at(ram_obj, 0, Permission::writable());

    NonNull::new(vaddr)
}

pub fn derive_cap<T: KernelObject>(cap: &Capability<T>) -> Option<Capability<T>> {
    let copy_cap_slot = gsm!().cspace_alloc()?;
    cap.derive(copy_cap_slot).ok()?;
    Some(Capability::<T>::new(copy_cap_slot))
}

pub fn copy_cap<T: KernelObject>(src: &Capability<T>) -> Option<Capability<T>> {
        let copy_slot = gsm!().cspace_alloc()?;
        let cspace = CNodeCap::new(rustyl4api::init::InitCSpaceSlot::InitCSpace as usize);
        cspace.cap_copy(copy_slot, src.slot).ok()?;
        Some(Capability::<T>::new(copy_slot))
}