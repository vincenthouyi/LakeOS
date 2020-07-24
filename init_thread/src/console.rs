use alloc::collections::linked_list::LinkedList;

use mutex::Mutex;
use pi::uart::{MiniUart, IrqStatus};

pub struct Console {
    inner: Option<MiniUart>,
    tx_buf: LinkedList<u8>,
    rx_buf: LinkedList<u8>,
}

impl Console {
    pub const fn new() -> Console {
        Console { inner: None, tx_buf: LinkedList::new(), rx_buf: LinkedList::new() }
    }

    pub fn initialize(&mut self, mini_uart: MiniUart) {
        if let None = self.inner {
            self.inner = Some(mini_uart);
        }
    }

    pub fn try_read_byte(&mut self) -> Option<u8> {
        self.rx_buf.pop_front()
    }

    pub fn write_byte(&mut self, byte: u8) {
        let was_empty = self.tx_buf.is_empty();
        self.tx_buf.push_back(byte);

        if let Some(inner) = self.inner.as_mut() {
            if was_empty {
                inner.enable_tx_irq();
            }
        }
    }

    pub fn read_from_inner(&mut self) {
        if let Some(inner) = self.inner.as_mut() {
            if inner.has_byte() {
                let c = inner.read_byte();
                self.rx_buf.push_back(c);
            }
        }
    }

    pub fn write_to_inner(&mut self) {
        if let Some(inner) = self.inner.as_mut() {
            // TODO: figure out why it lose some irq
//            if let Some(c) = self.tx_buf.pop_front() {
//                inner.write_byte(c);
//            } else {
//                inner.disable_tx_irq();
//            }
            while let Some(c) = self.tx_buf.pop_front() {
                inner.write_byte(c);
            }
            inner.disable_tx_irq();
        }
    }

    pub fn enable_tx_irq(&mut self) {
        if let Some(inner) = self.inner.as_mut() {
            inner.enable_tx_irq();
        }
    }

    pub fn enable_rx_irq(&mut self) {
        if let Some(inner) = self.inner.as_mut() {
            inner.enable_rx_irq();
        }
    }

    pub fn irq_status(&mut self) -> IrqStatus {
        if let Some(inner) = self.inner.as_mut() {
            inner.irq_status()
        } else {
            IrqStatus::Clear
        }
    }
}

impl core::fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(byte);
        }
        Ok(())
    }
}

pub static CONSOLE: Mutex<Console> = Mutex::new(Console::new());

pub fn console_main() -> ! {
    use rustyl4api::object::{Capability, EndpointObj, InterruptObj};
    use naive::space_manager::gsm;

    let irq_ep = gsm!().alloc_object::<EndpointObj>(12).unwrap();
    let irq_cntl_cap = Capability::<InterruptObj>::new(rustyl4api::init::InitCSpaceSlot::IrqController as usize);
    irq_cntl_cap.attach_ep_to_irq(irq_ep.slot, pi::interrupt::Interrupt::Aux as usize).unwrap();

    {
        let mut con = CONSOLE.lock();
        if con.tx_buf.is_empty() {
            con.enable_tx_irq();
        }

        con.enable_rx_irq();
    }

    loop {
        irq_ep.receive(None).unwrap();
        let mut con = CONSOLE.lock();

        loop {
            match con.irq_status() {
                IrqStatus::Tx => { con.write_to_inner() }
                IrqStatus::Rx => { con.read_from_inner() }
                IrqStatus::Clear => { break }
            }
        }
    }
}

pub fn init_console_server() {
    use crate::gpio;
    use pi::gpio::Function;

    gpio::GPIO_SERVER.lock().as_mut().unwrap().get_pin(14).unwrap().into_alt(Function::Alt5);
    gpio::GPIO_SERVER.lock().as_mut().unwrap().get_pin(15).unwrap().into_alt(Function::Alt5);

    let uart_base = naive::space_manager::allocate_frame_at(0x3f215000, 4096).unwrap();
    let mut uart = MiniUart::new(uart_base.as_ptr() as usize);
    uart.initialize(115200);

    CONSOLE.lock().initialize(uart);

    naive::thread::spawn(console_main);
}

pub fn console_read_byte() -> u8 {
    loop {
        if let Some(b) = CONSOLE.lock().try_read_byte() {
            return b;
        }
    }
}

pub fn console_print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    CONSOLE.lock().write_fmt(args).unwrap();
}

pub macro println {
    () => (print!("\n")),
    ($fmt:expr) => (print!(concat!($fmt, "\n"))),
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*))
}

pub macro print($($arg:tt)*) {
    console_print(format_args!($($arg)*))
}