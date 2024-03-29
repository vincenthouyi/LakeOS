use crate::objects::CapSlot;
use crate::space_manager::gsm;
use log::{set_logger, set_max_level, LevelFilter};

const MEMPOOL_SIZE: usize = usize::pow(2, 15);
const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Trace;

#[repr(align(4096))]
struct InitMemPool([u8; MEMPOOL_SIZE]);
static mut INIT_ALLOC_MEMPOOL: InitMemPool = InitMemPool([0u8; MEMPOOL_SIZE]);
static mut INIT_ALLOC_BACKUP_MEMPOOL: InitMemPool = InitMemPool([0u8; MEMPOOL_SIZE]);

pub fn populate_app_cspace() {
    use crate::objects::identify::{cap_identify, IdentifyResult};
    use rustyl4api::process::{ProcessCSpace, PROCESS_ROOT_CNODE_SIZE};

    let mut cap_max = 1;
    for i in ProcessCSpace::WellKnownMax as usize..PROCESS_ROOT_CNODE_SIZE {
        let res = cap_identify(i).unwrap();
        if let IdentifyResult::NullObj = res {
            cap_max = i;
            break;
        }
        let slot = gsm!().cspace_alloc_at(i);
        core::mem::forget(slot);
    }

    for i in 1..cap_max {
        let res = cap_identify(i).unwrap();
        let slot = CapSlot::new(i);
        gsm!().insert_identify(slot, res);
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
        crate::vm_allocator::GLOBAL_VM_ALLOC.add_mempool(
            INIT_ALLOC_MEMPOOL.0.as_ptr() as *mut u8,
            INIT_ALLOC_MEMPOOL.0.len(),
        );
        crate::vm_allocator::GLOBAL_VM_ALLOC.add_backup_mempool(
            INIT_ALLOC_BACKUP_MEMPOOL.0.as_ptr() as *mut u8,
            INIT_ALLOC_BACKUP_MEMPOOL.0.len(),
        );
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

    set_logger(&rustyl4api::debug_printer::DEBUG_PRINTER)
        .map(|()| set_max_level(DEFAULT_LOG_LEVEL))
        .unwrap();

    initialize_mm();

    populate_app_cspace();

    initialize_vmspace();

    unsafe {
        main();
    }
}
