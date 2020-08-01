use crate::utils::MASK;

fn tpidrro_el0() -> usize {
    let tpidrro: usize;

    unsafe{
        llvm_asm!("mrs $0, tpidrro_el0" : "=r"(tpidrro));
    }
    
    tpidrro
}

pub fn thread_id() -> usize {
    tpidrro_el0() & MASK!(48)
}

pub fn cpu_id() -> usize {
    tpidrro_el0() >> 48
}