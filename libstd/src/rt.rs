use rustyl4api::kprintln;
use crate::space_manager::{gsm, gsm_init};

extern "Rust" {
    fn main();
}

const MEMPOOL_SIZE: usize = 4096;

#[repr(align(4096))]
struct InitMemPool([u8; MEMPOOL_SIZE]);
static mut INIT_ALLOC_MEMPOOL: InitMemPool = InitMemPool([0u8; MEMPOOL_SIZE]);
static mut INIT_ALLOC_BACKUP_MEMPOOL: InitMemPool = InitMemPool([0u8; MEMPOOL_SIZE]);

fn populate_init_cspace() {
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
//        debug_println!("ret cap[{}]: {:x?}", i, res);
        gsm!().cspace_alloc_at(i);
    }

    let untyped_idx = ProcessCSpace::InitUntyped as usize;
    let res = cap_identify(untyped_idx).unwrap();

    gsm!().insert_identify(untyped_idx, res);
}

fn initialize_vmspace() {
    use rustyl4api::vspace::{FRAME_SIZE};

    let brk = unsafe{ crate::_end.as_ptr() as usize };
    let brk = crate::utils::align_up(brk, FRAME_SIZE);

    gsm!().insert_vm_range(0, brk);
}

fn app_cpu_entry() {
    kprintln!("CPU {} in user space!", rustyl4api::thread::thread_id());

    loop {}
}
#[no_mangle]
pub fn _start() -> ! {

    unsafe {
        crate::vm_allocator::GLOBAL_VM_ALLOC
            .add_mempool(INIT_ALLOC_MEMPOOL.0.as_ptr() as *mut u8,
                         INIT_ALLOC_MEMPOOL.0.len());
        crate::vm_allocator::GLOBAL_VM_ALLOC
            .add_backup_mempool(INIT_ALLOC_BACKUP_MEMPOOL.0.as_ptr() as *mut u8,
                         INIT_ALLOC_BACKUP_MEMPOOL.0.len());
    }

    populate_init_cspace();

    unsafe { main(); }
    loop {}
//    unreachable!("Init Returns!");
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
//    debug_println!("Panic! {:?}", _info);
    loop {
    }
}

// pub trait Termination {
//     /// Is called to get the representation of the value as status code.
//     /// This status code is returned to the operating system.
//     fn report(self) -> i32;
// }

// #[lang = "start"]
// fn start<T: Termination + 'static>(main: fn() -> T, _: isize, _: *const *const u8) -> isize {
//     main().report() as isize
// }
