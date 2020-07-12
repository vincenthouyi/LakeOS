use rustyl4api::debug_printer::kprintln;
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

fn initialize_vmspace() {
    use rustyl4api::vspace::{FRAME_SIZE};

    let brk = unsafe{ crate::_end.as_ptr() as usize };
    let brk = crate::utils::align_up(brk, FRAME_SIZE);

    gsm!().insert_vm_range(0, brk);
}

fn run_app_cpus() {
    use rustyl4api::object::{Capability, TcbObj, MonitorObj};
    use rustyl4api::init::InitCSpaceSlot::{InitL1PageTable,InitCSpace, Monitor};
    use rustyl4api::vspace::{Permission, FRAME_SIZE};

    for i in 1 .. 4 {
        let init_tcb = gsm!().alloc_object::<TcbObj>(12)
                             .unwrap();
        let stack_base = gsm!().map_frame_at(0, 0, FRAME_SIZE, Permission::writable()).unwrap();
        init_tcb.configure(InitL1PageTable as usize, InitCSpace as usize)
        .expect("Error Configuring TCB");
        init_tcb.set_registers(0b1100, app_cpu_entry as usize, stack_base as usize + FRAME_SIZE)
        .expect("Error Setting Registers");
        

        let monitor_cap = Capability::<MonitorObj>::new(Monitor as usize);
        monitor_cap.insert_tcb_to_cpu(&init_tcb, i).unwrap();
    }
}

fn app_cpu_entry() {
    kprintln!("CPU {} in user space!", rustyl4api::thread::thread_id());

    loop {}
}

#[no_mangle]
pub fn _start() -> ! {
    kprintln!("赞美太阳！");

    unsafe {
        crate::vm_allocator::GLOBAL_VM_ALLOC
            .add_mempool(INIT_ALLOC_MEMPOOL.0.as_ptr() as *mut u8,
                         INIT_ALLOC_MEMPOOL.0.len());
        crate::vm_allocator::GLOBAL_VM_ALLOC
            .add_backup_mempool(INIT_ALLOC_BACKUP_MEMPOOL.0.as_ptr() as *mut u8,
                         INIT_ALLOC_BACKUP_MEMPOOL.0.len());
    }

    populate_init_cspace();

    initialize_vmspace();

    run_app_cpus();

    unsafe { main(); }
    unreachable!("Init Returns!");
}
