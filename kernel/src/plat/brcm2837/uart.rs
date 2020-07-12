use core::fmt::{Arguments, Write};

use spin::Mutex;
use pi::uart::MiniUart;
/// The base address for the `MU` registers.
const MU_REG_PAGE_BASE: usize = crate::prelude::IO_BASE + 0x215000;
static DEBUG_CONSOLE: Mutex<Option<MiniUart>> = Mutex::new(None);

pub fn console_print(args: Arguments) {
    DEBUG_CONSOLE
        .lock()
        .as_mut()
        .unwrap()
        .write_fmt(args)
        .unwrap()
}

pub fn init_uart() {
    use pi::gpio::{Gpio, Function};

    /* Switch GPIO 14 and 15 mode to Alt5 */
    Gpio::new(14, crate::prelude::IO_BASE + 0x200000).into_alt(Function::Alt5);
    Gpio::new(15, crate::prelude::IO_BASE + 0x200000).into_alt(Function::Alt5);

    let mut console = DEBUG_CONSOLE.lock();
    *console = Some(MiniUart::new(MU_REG_PAGE_BASE));
    console.as_mut().unwrap().initialize(115200);
}