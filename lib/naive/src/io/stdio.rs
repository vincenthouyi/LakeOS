use core::fmt;
use core::pin::Pin;
use core::task::{Context, Poll};

use alloc::{format, sync::Arc};

use spin::Mutex;

use conquer_once::spin::OnceCell;

use futures_util::io::{AsyncRead, AsyncWrite};
use futures_util::stream::Stream;

use crate::objects::EpCap;
use rustyl4api::process::ProcessCSpace;

use crate::fs::File;
use crate::io;

pub struct Stdout {
    fd: Arc<Mutex<File>>,
}

impl Stdout {
    pub fn new(fd: Arc<Mutex<File>>) -> Self {
        Self { fd }
    }
}

impl AsyncWrite for Stdout {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.fd.lock()).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

static STDOUT: OnceCell<Arc<Mutex<File>>> = OnceCell::uninit();

pub async fn stdout() -> Stdout {
    if !STDOUT.is_initialized() {
        let fd = File::connect(EpCap::new(ProcessCSpace::Stdout as usize))
            .await
            .unwrap();
        STDOUT.get_or_init(|| Arc::new(Mutex::new(fd)));
    }
    let inner = STDOUT.get().unwrap().clone();
    Stdout::new(inner)
}

pub fn set_stdout(file_handle: File) {
    if !STDOUT.is_initialized() {
        STDOUT.get_or_init(|| Arc::new(Mutex::new(file_handle)));
    } else {
        *STDOUT.get().unwrap().lock() = file_handle;
    }
}

pub struct Stdin {
    fd: Arc<Mutex<File>>,
}

impl Stdin {
    pub fn new(fd: Arc<Mutex<File>>) -> Self {
        Self { fd }
    }
}

impl AsyncRead for Stdin {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.fd.lock()).poll_read(cx, buf)
    }
}

impl Stream for Stdin {
    type Item = u8;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buf = [0u8; 1];
        self.poll_read(cx, &mut buf).map(|r| r.ok().map(|_| buf[0]))
    }
}

static STDIN: OnceCell<Arc<Mutex<File>>> = OnceCell::uninit();

pub async fn stdin() -> Stdin {
    if !STDIN.is_initialized() {
        let fd = File::connect(EpCap::new(ProcessCSpace::Stdin as usize))
            .await
            .unwrap();
        STDIN.get_or_init(|| Arc::new(Mutex::new(fd)));
    }
    let inner = STDIN.get().unwrap().clone();
    Stdin::new(inner)
}

pub fn set_stdin(file_handle: File) {
    if !STDIN.is_initialized() {
        STDIN.get_or_init(|| Arc::new(Mutex::new(file_handle)));
    } else {
        *STDIN.get().unwrap().lock() = file_handle;
    }
}

pub async fn _print(args: fmt::Arguments<'_>) {
    use futures_util::io::AsyncWriteExt;

    if let Err(e) = stdout()
        .await
        .write_all(&format!("{}", args).into_bytes())
        .await
    {
        panic!("failed printing to stdout: {}", e);
    }
}
