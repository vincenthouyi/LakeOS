use crate::debug_printer::*;
use crate::space_manager::INIT_ALLOC;

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
    use rustyl4api::object::{Capability, identify, VTableObj, RamObj};
    use rustyl4api::vspace::{FRAME_SIZE};
    use rustyl4api::object::identify::IdentifyResult;

    let brk = unsafe{ crate::_end.as_ptr() as usize };
    let brk = crate::utils::align_up(brk, FRAME_SIZE);

    let root_cnode = Capability::new(InitCSpaceSlot::InitCSpace as usize);
    let root_vnode = Capability::new(InitCSpaceSlot::InitL1PageTable as usize);

    INIT_ALLOC.initialize(root_cnode, INIT_CSPACE_SIZE, root_vnode, brk);

    INIT_ALLOC.cspace_alloc_at(0);
    for i in 1 .. {
        let res = identify::cap_identify(i).unwrap();
        if let IdentifyResult::NullObj = res {
            break;
        }
//        debug_println!("ret cap[{}]: {:x?}", i, res);
        INIT_ALLOC.cspace_alloc_at(i);
    }

    for i in InitCSpaceSlot::UntypedStart as usize .. INIT_CSPACE_SIZE {
        let res = identify::cap_identify(i).unwrap();

        if let IdentifyResult::NullObj = res {
            break;
        }

        match res {
            IdentifyResult::VTable{mapped_vaddr, mapped_asid, level} => {
                let table = Capability::<VTableObj>::new(i);
                INIT_ALLOC.insert_vtable(table, mapped_vaddr, level - 1);
            }
            IdentifyResult::Ram {bit_sz, mapped_vaddr, mapped_asid, is_device} => {
                let cap = Capability::<RamObj>::new(i);

                INIT_ALLOC.install_ram(cap, mapped_vaddr);
            }
            _ => { }
        }
    }

    for i in InitCSpaceSlot::UntypedStart as usize .. INIT_CSPACE_SIZE {
        let res = identify::cap_identify(i).unwrap();
        if let IdentifyResult::NullObj = res {
            break;
        }
        match res {
            IdentifyResult::Untyped{paddr, bit_sz, is_device, free_offset} => {
                INIT_ALLOC.insert_untyped(i, paddr, bit_sz, is_device, free_offset);
            }
            _ => { }
        }
    }
}

#[no_mangle]
pub fn _start() -> ! {
    debug_println!("赞美太阳！");

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
    unreachable!("Init Returns!");
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    debug_println!("Panic! {:?}", _info);
    loop {
    }
}