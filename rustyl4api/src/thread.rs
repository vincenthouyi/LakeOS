

pub fn thread_id() -> usize {
    let tid: usize;

    unsafe{
        llvm_asm!("mrs $0, tpidrro_el0" : "=r"(tid));
    }

    tid
}