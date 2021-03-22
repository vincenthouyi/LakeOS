use core::task::Waker;
use core::{
    pin::Pin,
    task::{Context, Poll},
};

use alloc::collections::LinkedList;
use alloc::sync::Arc;

use futures_util::io::{AsyncRead, AsyncWrite};
use futures_util::stream::Stream;
use spin::Mutex;

use naive::ep_server::{EpNtfHandler, EpServer};
use naive::io;
use pi::uart::{IrqStatus, MiniUart};

pub struct Console {
    inner: MiniUart,
    rx_waker: LinkedList<Waker>,
    tx_waker: LinkedList<Waker>,
}
impl Console {
    pub fn new(mini_uart: MiniUart) -> Console {
        Console {
            inner: mini_uart,
            tx_waker: LinkedList::new(),
            rx_waker: LinkedList::new(),
        }
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
    inner: Arc<Mutex<Console>>,
}

impl ConsoleExt {
    pub fn new(console: Console) -> Self {
        Self {
            inner: Arc::new(Mutex::new(console)),
        }
    }
}

impl EpNtfHandler for ConsoleExt {
    fn handle_notification(&self, _ep_server: &EpServer, _ntf: usize) {
        let mut inner = self.inner.lock();
        match inner.irq_status() {
            IrqStatus::Rx => {
                while let Some(waker) = inner.rx_waker.pop_front() {
                    waker.wake();
                }
                inner.disable_rx_irq();
            }
            IrqStatus::Tx => {
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

pub async fn console_server_init() {
    use crate::gpio;
    use naive::space_manager::gsm;
    use pi::gpio::Function;
    use rustyl4api::vspace::Permission;

    gpio::GPIO_SERVER
        .lock()
        .as_mut()
        .unwrap()
        .get_pin(14)
        .unwrap()
        .into_alt(Function::Alt5);
    gpio::GPIO_SERVER
        .lock()
        .as_mut()
        .unwrap()
        .get_pin(15)
        .unwrap()
        .into_alt(Function::Alt5);

    let uart_ram_cap = crate::request_memory(0x3f215000, 4096, true).await.unwrap();
    let uart_base = gsm!().insert_ram_at(uart_ram_cap, 0, Permission::writable());

    let mut uart = MiniUart::new(uart_base as usize);
    uart.initialize(115200);

    let mut con = Console::new(uart);
    con.disable_rx_irq();
    con.disable_tx_irq();
    let con = ConsoleExt::new(con);

    *CONSOLE.lock() = Some(con.clone());
}

pub fn console() -> ConsoleExt {
    loop {
        if let Some(c) = &*CONSOLE.lock() {
            return c.clone();
        }
    }
}

impl AsyncWrite for ConsoleExt {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let mut write_len = 0;
        let mut inner = self.inner.lock();
        for b in buf {
            if inner.can_write() {
                inner.write_byte(*b);
                write_len += 1;
            } else {
                break;
            }
        }

        if write_len == 0 {
            inner.tx_waker.push_back(cx.waker().clone());
            inner.enable_tx_irq();
            Poll::Pending
        } else {
            Poll::Ready(Ok(write_len))
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

impl AsyncRead for ConsoleExt {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let mut read_len = 0;
        let mut inner = self.inner.lock();
        while read_len < buf.len() {
            if inner.can_read() {
                buf[read_len] = inner.read_byte();
                read_len += 1;
            } else {
                break;
            }
        }

        if read_len == 0 {
            inner.rx_waker.push_back(cx.waker().clone());
            inner.enable_rx_irq();
            Poll::Pending
        } else {
            Poll::Ready(Ok(read_len))
        }
    }
}

impl Stream for ConsoleExt {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let mut buf = [0u8; 1];
        self.poll_read(cx, &mut buf).map(|r| r.ok().map(|_| buf[0]))
    }
}
