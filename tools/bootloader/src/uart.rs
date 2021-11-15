use core::fmt::{Arguments, Write};
use log::{set_logger, Level, LevelFilter, Log, Metadata, Record};

use pi::uart::MiniUart;
use spin::Mutex;
const IO_BASE: usize = 0x3f000000;
/// The base address for the `MU` registers.
const MU_REG_PAGE_BASE: usize = IO_BASE + 0x215000;
static DEBUG_CONSOLE: Console = Console::new(None);

struct Console(Mutex<Option<MiniUart>>);

impl Console {
    pub const fn new(inner: Option<MiniUart>) -> Self {
        Self(Mutex::new(inner))
    }
}

pub fn console_print(args: Arguments) {
    DEBUG_CONSOLE
        .0
        .lock()
        .as_mut()
        .unwrap()
        .write_fmt(args)
        .unwrap()
}

pub fn init_uart() {
    use pi::gpio::{Function, Gpio};

    /* Switch GPIO 14 and 15 mode to Alt5 */
    Gpio::new(14, IO_BASE + 0x200000).into_alt(Function::Alt5);
    Gpio::new(15, IO_BASE + 0x200000).into_alt(Function::Alt5);

    let mut console = DEBUG_CONSOLE.0.lock();
    *console = Some(MiniUart::new(MU_REG_PAGE_BASE));
    console.as_mut().unwrap().initialize(115200);
    set_logger(&DEBUG_CONSOLE)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .unwrap();
}

impl Log for Console {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            kprintln!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
