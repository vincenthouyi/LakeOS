/// The base address for the `MU` registers.
const MU_REG_PAGE_BASE: usize = crate::prelude::IO_BASE + 0x215000;

pub fn console_print(args: core::fmt::Arguments) {
    use core::fmt::Write;

    pi::uart::MiniUart::new(MU_REG_PAGE_BASE)
        .write_fmt(args)
        .unwrap();
}

pub fn init_uart() {
    use pi::gpio::{Gpio, Function};
    use pi::uart::MiniUart;

    /* Switch GPIO 14 and 15 mode to Alt5 */
    Gpio::new(14, crate::prelude::IO_BASE + 0x200000).into_alt(Function::Alt5);
    Gpio::new(15, crate::prelude::IO_BASE + 0x200000).into_alt(Function::Alt5);

    MiniUart::new(MU_REG_PAGE_BASE).initialize(115200);
}