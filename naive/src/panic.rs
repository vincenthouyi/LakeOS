use rustyl4api::kprintln;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    kprintln!("Panic! {:?}", _info);
    loop {
    }
}