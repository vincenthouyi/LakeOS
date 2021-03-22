use core::num::NonZeroUsize;

use rustyl4api::process::{ProcessCSpace, PROCESS_ROOT_CNODE_SIZE};

use crate::objects::KernelObject;
use crate::objects::{Capability, EpRef, CNodeRef, VTableRef};
use crate::spaceman::SpaceManager;

lazy_static! {
    pub static ref GLOBAL_SPACEMAN: SpaceManager = {
        SpaceManager::new(ROOT_CNODE_CAP.clone(), PROCESS_ROOT_CNODE_SIZE, ROOT_VNODE_CAP.clone())
    };
}

pub macro gsm() {
    {
        if crate::vm_allocator::GLOBAL_VM_ALLOC.cur_pool_remain() < 3072 {
            use rustyl4api::vspace::{Permission, FRAME_SIZE};

            let addr = GLOBAL_SPACEMAN
                .map_frame_at(0, 0, FRAME_SIZE, Permission::writable())
                .unwrap();
            crate::vm_allocator::GLOBAL_VM_ALLOC.add_mempool(addr, FRAME_SIZE);
        }
        &GLOBAL_SPACEMAN
    }
}

pub fn copy_cap<T: KernelObject>(src: &Capability<T>) -> Option<Capability<T>> {
    copy_cap_badged(src, None)
}

pub fn copy_cap_badged<T: KernelObject>(src: &Capability<T>, badge: Option<NonZeroUsize>) -> Option<Capability<T>> {
    let copy_slot = gsm!().cspace_alloc()?;
    gsm!().root_cnode().cap_copy_badged(copy_slot.slot(), src.slot.slot(), badge).ok()?;
    Some(Capability::<T>::new(copy_slot))
}

lazy_static! {
    pub static ref NAME_SERVICE_CAP: EpRef = {
        EpRef::from_slot_num(ProcessCSpace::NameServer as usize)
    };
    pub static ref STDIN_CAP: EpRef = {
        EpRef::from_slot_num(ProcessCSpace::Stdin as usize)
    };
    pub static ref STDOUT_CAP: EpRef = {
        EpRef::from_slot_num(ProcessCSpace::Stdout as usize)
    };
    pub static ref STDERR_CAP: EpRef = {
        EpRef::from_slot_num(ProcessCSpace::Stderr as usize)
    };
    pub static ref ROOT_VNODE_CAP: VTableRef = {
        VTableRef::from_slot_num(ProcessCSpace::RootVNodeCap as usize)
    };
    pub static ref ROOT_CNODE_CAP: CNodeRef = {
        CNodeRef::from_slot_num(ProcessCSpace::RootCNodeCap as usize)
    };
}