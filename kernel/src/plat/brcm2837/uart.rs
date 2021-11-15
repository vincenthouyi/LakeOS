use core::fmt::{Arguments, Write};
use log::{set_logger, set_max_level, LevelFilter, Log, Metadata, Record};

use pi::uart::MiniUart;
use spin::Mutex;
/// The base address for the `MU` registers.
const MU_REG_PAGE_BASE: usize = crate::prelude::IO_BASE + 0x215000;
const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Trace;
static DEBUG_CONSOLE: Console = Console::new(None);

struct Console(Mutex<Option<MiniUart>>);

impl Console {
    pub const fn new(inner: Option<MiniUart>) -> Self {
        Self(Mutex::new(inner))
    }
}

impl Log for Console {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        self.0.lock().is_some()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let _ = self.0.lock().as_mut().map(|con| {
                con.write_fmt(format_args!(
                    "[Kernel:{}-{}] {}\n",
                    crate::arch::cpuid(),
                    record.level(),
                    record.args()
                ))
                .expect("fail to print");
            });
        }
    }

    fn flush(&self) {}
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
    Gpio::new(14, crate::prelude::IO_BASE + 0x200000).into_alt(Function::Alt5);
    Gpio::new(15, crate::prelude::IO_BASE + 0x200000).into_alt(Function::Alt5);

    let mut console = DEBUG_CONSOLE.0.lock();
    *console = Some(MiniUart::new(MU_REG_PAGE_BASE));
    console.as_mut().unwrap().initialize(115200);
    set_logger(&DEBUG_CONSOLE)
        .map(|()| set_max_level(DEFAULT_LOG_LEVEL))
        .unwrap();
}
