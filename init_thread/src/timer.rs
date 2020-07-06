use pi::timer::Timer;
use mutex::Mutex;

pub static SYSTEM_TIMER: Mutex<Option<Timer>> = Mutex::new(None);

pub fn init_timer_server() {
    let timer_base = naive::space_manager::allocate_frame_at(0x3F003000, 4096).unwrap();

    *SYSTEM_TIMER.lock() = Some(Timer::new(timer_base.as_ptr() as usize));
}

/// Returns the current time in microseconds.
pub fn current_time() -> u64 {
    SYSTEM_TIMER.lock().as_ref().unwrap().read()
}

/// Spins until `us` microseconds have passed.
pub fn spin_sleep_us(us: u64) {
    let old = current_time();
    loop {
        let new = current_time();
        if old + us <= new {
            break;
        }
    }
}

/// Spins until `ms` milliseconds have passed.
pub fn spin_sleep_ms(ms: u64) {
    spin_sleep_us(ms * 1000);
}

/// Sets up a match in timer 1 to occur `us` microseconds from now. If
/// interrupts for timer 1 are enabled and IRQs are unmasked, then a timer
/// interrupt will be issued in `us` microseconds.
pub fn tick_in(us: u32) {
    SYSTEM_TIMER.lock().as_mut().unwrap().tick_in(us)
}
