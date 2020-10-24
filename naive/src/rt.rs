use crate::space_manager::{gsm, gsm_init};

const MEMPOOL_SIZE: usize = 4096;

#[repr(align(4096))]
struct InitMemPool([u8; MEMPOOL_SIZE]);
static mut INIT_ALLOC_MEMPOOL: InitMemPool = InitMemPool([0u8; MEMPOOL_SIZE]);
static mut INIT_ALLOC_BACKUP_MEMPOOL: InitMemPool = InitMemPool([0u8; MEMPOOL_SIZE]);

pub fn populate_app_cspace() {
    use rustyl4api::process::{ProcessCSpace, PROCESS_ROOT_CNODE_SIZE};
    use rustyl4api::object::Capability;
    use rustyl4api::object::identify::{cap_identify, IdentifyResult};

    let root_cnode = Capability::new(ProcessCSpace::RootCNodeCap as usize);
    let root_vnode = Capability::new(ProcessCSpace::RootVNodeCap as usize);

    gsm_init(root_cnode, PROCESS_ROOT_CNODE_SIZE, root_vnode);

    gsm!().cspace_alloc_at(0);

    let mut cap_max = 1;
    for i in ProcessCSpace::ProcessFixedMax as usize .. PROCESS_ROOT_CNODE_SIZE {
        let res = cap_identify(i).unwrap();
        if let IdentifyResult::NullObj = res {
            cap_max = i;
            break;
        }
        gsm!().cspace_alloc_at(i);
    }

    for i in 1 .. cap_max {
        let res = cap_identify(i).unwrap();

        gsm!().insert_identify(i, res);
    }
}

pub fn initialize_vmspace() {
    // use rustyl4api::vspace::{FRAME_SIZE};

    // let brk = unsafe{ crate::_end.as_ptr() as usize };
    // let brk = crate::utils::align_up(brk, FRAME_SIZE);

    // gsm!().insert_vm_range(0, brk);
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

extern "C" {
    static mut __bss_start__: [u8; 0];
    static mut __bss_end__: [u8; 0];
}

extern "Rust" {
    fn main() -> !;
}

#[no_mangle]
pub fn _start() -> ! {
    unsafe {
        r0::zero_bss(__bss_start__.as_mut_ptr(), __bss_end__.as_mut_ptr());
    }

    initialize_mm();

    populate_app_cspace();

    initialize_vmspace();

    unsafe {
        main();
    }
}