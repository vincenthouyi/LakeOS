pub fn idle_thread() {
    loop {
        super::wfi();
    }
}
