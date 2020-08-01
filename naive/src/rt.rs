use crate::space_manager::{gsm, gsm_init};

const MEMPOOL_SIZE: usize = 4096;

#[repr(align(4096))]
struct InitMemPool([u8; MEMPOOL_SIZE]);
static mut INIT_ALLOC_MEMPOOL: InitMemPool = InitMemPool([0u8; MEMPOOL_SIZE]);
static mut INIT_ALLOC_BACKUP_MEMPOOL: InitMemPool = InitMemPool([0u8; MEMPOOL_SIZE]);

pub fn populate_init_cspace() {
    use rustyl4api::init::{InitCSpaceSlot, INIT_CSPACE_SIZE};
    use rustyl4api::object::Capability;
    use rustyl4api::object::identify::{cap_identify, IdentifyResult};

    let root_cnode = Capability::new(InitCSpaceSlot::InitCSpace as usize);
    let root_vnode = Capability::new(InitCSpaceSlot::InitL1PageTable as usize);

    gsm_init(root_cnode, INIT_CSPACE_SIZE, root_vnode);

    gsm!().cspace_alloc_at(0);

    let mut cap_max = 1;
    for i in 1 .. INIT_CSPACE_SIZE {
        let res = cap_identify(i).unwrap();
        if let IdentifyResult::NullObj = res {
            cap_max = i;
            break;
        }
//        debug_println!("ret cap[{}]: {:x?}", i, res);
        gsm!().cspace_alloc_at(i);
    }

    for i in InitCSpaceSlot::UntypedStart as usize .. cap_max {
        let res = cap_identify(i).unwrap();

        if let IdentifyResult::NullObj = res {
            break;
        }

        gsm!().insert_identify(i, res);
    }
}

pub fn populate_app_cspace() {
    use rustyl4api::process::{ProcessCSpace, PROCESS_ROOT_CNODE_SIZE};
    use rustyl4api::object::Capability;
    use rustyl4api::object::identify::{cap_identify, IdentifyResult};

    let root_cnode = Capability::new(ProcessCSpace::RootCNodeCap as usize);
    let root_vnode = Capability::new(ProcessCSpace::RootVNodeCap as usize);

    gsm_init(root_cnode, PROCESS_ROOT_CNODE_SIZE, root_vnode);

    gsm!().cspace_alloc_at(0);

    let mut cap_max = 1;
    for i in 1 .. PROCESS_ROOT_CNODE_SIZE {
        let res = cap_identify(i).unwrap();
        if let IdentifyResult::NullObj = res {
            cap_max = i;
            break;
        }
        gsm!().cspace_alloc_at(i);
    }

    for i in ProcessCSpace::ProcessFixedMax as usize .. cap_max {
        let res = cap_identify(i).unwrap();

        // rustyl4api::kprintln!("ret cap[{}]: {:x?}", i, res);
        if let IdentifyResult::NullObj = res {
            break;
        }

        gsm!().insert_identify(i, res);
    }

    let untyped_idx = ProcessCSpace::InitUntyped as usize;
    let res = cap_identify(untyped_idx).unwrap();

    gsm!().insert_identify(untyped_idx, res);
}

pub fn initialize_vmspace() {
    use rustyl4api::vspace::{FRAME_SIZE};

    let brk = unsafe{ crate::_end.as_ptr() as usize };
    let brk = crate::utils::align_up(brk, FRAME_SIZE);

    gsm!().insert_vm_range(0, brk);
}

pub fn initialize_mm() {
    unsafe {
        crate::vm_allocator::GLOBAL_VM_ALLOC
            .add_mempool(INIT_ALLOC_MEMPOOL.0.as_ptr() as *mut u8,
                         INIT_ALLOC_MEMPOOL.0.len());
        crate::vm_allocator::GLOBAL_VM_ALLOC
            .add_backup_mempool(INIT_ALLOC_BACKUP_MEMPOOL.0.as_ptr() as *mut u8,
                         INIT_ALLOC_BACKUP_MEMPOOL.0.len());
    }
}