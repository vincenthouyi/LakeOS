use rustyl4api::kprintln;

extern "Rust" {
    fn main();
}

#[no_mangle]
pub fn _start() -> ! {
    kprintln!("赞美太阳！");
//
//    unsafe {
//        crate::vm_allocator::GLOBAL_VM_ALLOC
//            .add_mempool(INIT_ALLOC_MEMPOOL.0.as_ptr() as *mut u8,
//                         INIT_ALLOC_MEMPOOL.0.len());
//        crate::vm_allocator::GLOBAL_VM_ALLOC
//            .add_backup_mempool(INIT_ALLOC_BACKUP_MEMPOOL.0.as_ptr() as *mut u8,
//                         INIT_ALLOC_BACKUP_MEMPOOL.0.len());
//    }
//
//    populate_init_cspace();
//
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
