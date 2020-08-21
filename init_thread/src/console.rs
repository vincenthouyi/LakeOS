use core::future::Future;
use core::{pin::Pin, task::{Poll, Context}};
use core::task::Waker;

use alloc::collections::LinkedList;
use alloc::sync::Arc;

use futures_util::stream::Stream;
use spin::Mutex;

use pi::uart::{MiniUart, IrqStatus};
use naive::ep_server::{EpServer, EpNtfHandler};

pub struct Console {
    inner: MiniUart,
    rx_waker: LinkedList<Waker>,
    tx_waker: LinkedList<Waker>,
}
impl Console {
    pub const fn new(mini_uart: MiniUart) -> Console {
        Console { inner: mini_uart, tx_waker: LinkedList::new(), rx_waker: LinkedList::new() }
    }

    pub fn can_read(&self) -> bool {
        self.inner.has_byte()
    }

    pub fn can_write(&self) -> bool {
        self.inner.can_write()
    }

    pub fn read_byte(&mut self) -> u8 {
        self.inner.read_byte()
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.inner.write_byte(byte)
    }

    pub fn enable_tx_irq(&mut self) {
        self.inner.enable_tx_irq();
    }

    pub fn enable_rx_irq(&mut self) {
        self.inner.enable_rx_irq();
    }

    pub fn disable_tx_irq(&mut self) {
        self.inner.disable_tx_irq();
    }

    pub fn disable_rx_irq(&mut self) {
        self.inner.disable_rx_irq();
    }

    pub fn irq_status(&mut self) -> IrqStatus {
        self.inner.irq_status()
    }
}

#[derive(Clone)]
pub struct ConsoleExt {
    inner: Arc<Mutex<Console>>
}

impl ConsoleExt {
    pub fn new(console: Console) -> Self {
        Self { inner: Arc::new(Mutex::new(console)) }
    }

    pub fn stream(&self) -> ConsoleReader {
        ConsoleReader { inner: self.clone() }
    }

    pub fn can_read(&self) -> bool { self.inner.lock().can_read() }

    pub fn read_byte(&self) -> u8 { self.inner.lock().read_byte() }

    pub fn can_write(&self) -> bool { self.inner.lock().can_write() }

    pub fn write_byte(&self, byte: u8) { self.inner.lock().write_byte(byte) }

    pub fn poll_write<'a>(&self, buf: &'a [u8]) -> WriteFuture<'a> {
        WriteFuture {
            inner : self.clone(),
            buf: buf,
            write_len: 0,
        }
    }
}

impl EpNtfHandler for ConsoleExt {
    fn handle_notification(&self, ep_server: &EpServer, ntf: usize) {
        let mut inner = self.inner.lock();
        match inner.irq_status() {
            IrqStatus::Rx => {
                // TODO: find out why irq is generated when tx/rx irq disabled
                // if inner.tx_waker.is_empty() {
                //     kprintln!("rx_waker is empty");
                // }
                while let Some(waker) = inner.rx_waker.pop_front() {
                    waker.wake();
                }
                inner.disable_rx_irq();
            }
            IrqStatus::Tx => {
                // if inner.tx_waker.is_empty() {
                //     kprintln!("tx_waker is empty");
                // }
                while let Some(waker) = inner.tx_waker.pop_front() {
                    waker.wake();
                }
                inner.disable_tx_irq();
            }
            IrqStatus::Clear => {
                kprintln!("in clear");
            }
        }
    }
}

static CONSOLE: Mutex<Option<ConsoleExt>> = Mutex::new(None);

pub fn console_server() -> ConsoleExt {
    use crate::gpio;
    use pi::gpio::Function;

    gpio::GPIO_SERVER.lock().as_mut().unwrap().get_pin(14).unwrap().into_alt(Function::Alt5);
    gpio::GPIO_SERVER.lock().as_mut().unwrap().get_pin(15).unwrap().into_alt(Function::Alt5);

    let uart_base = naive::space_manager::allocate_frame_at(0x3f215000, 4096).unwrap();
    let mut uart = MiniUart::new(uart_base.as_ptr() as usize);
    uart.initialize(115200);

    let mut con = Console::new(uart);
    con.disable_rx_irq();
    con.disable_tx_irq();
    let con = ConsoleExt::new(con);

    *CONSOLE.lock() = Some(con.clone());

    con
}

pub fn console() -> ConsoleExt {
    loop {
        if let Some(c) = &*CONSOLE.lock() {
            return c.clone();
        }
    }
}

pub struct ConsoleReader {
    inner: ConsoleExt
}

impl Stream for ConsoleReader {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        if self.inner.can_read() {
            Poll::Ready(Some(self.inner.read_byte()))
        } else {
            let mut inner = self.inner.inner.lock();
            inner.rx_waker.push_back(cx.waker().clone());
            inner.enable_rx_irq();
            Poll::Pending
        }
    }
}

pub struct WriteFuture<'a> {
    inner: ConsoleExt,
    buf: &'a [u8],
    write_len: usize,
}

impl<'a> Future for WriteFuture<'a> {
    type Output = usize;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        while self.write_len < self.buf.len() {
            if !self.inner.can_write() {
                let mut inner = self.inner.inner.lock();
                inner.tx_waker.push_back(cx.waker().clone());
                inner.enable_tx_irq();
                return Poll::Pending;
            } else {
                let byte = self.buf[self.write_len];
                self.inner.write_byte(byte);
                self.write_len += 1;
            }
        }

        Poll::Ready(self.write_len)
    }
}