use naive::space_manager::{gsm};

extern "Rust" {
    fn main();
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
    kprintln!("CPU {} in user space!", rustyl4api::thread::cpu_id());

    loop {}
}

#[no_mangle]
pub fn _start() -> ! {
    kprintln!("赞美太阳！");

    naive::rt::initialize_mm();

    naive::rt::populate_init_cspace();

    naive::rt::initialize_vmspace();

    run_app_cpus();

    unsafe { main(); }
    unreachable!("Init Returns!");
}
