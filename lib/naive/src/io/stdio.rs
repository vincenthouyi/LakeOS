use core::fmt;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use alloc::{boxed::Box, format, sync::Arc};

use spin::Mutex;

use conquer_once::spin::OnceCell;

use futures_util::io::{AsyncRead, AsyncWrite};
use futures_util::stream::Stream;

use rustyl4api::object::EpCap;
use rustyl4api::process::ProcessCSpace;

use crate::ep_server::EP_SERVER;
use crate::io;
use crate::rpc::RpcClient;

pub struct Stdout {
    channel: Arc<Mutex<RpcClient>>,
}

impl Stdout {
    pub fn new(channel: Arc<Mutex<RpcClient>>) -> Self {
        Self { channel }
    }
}

impl AsyncWrite for Stdout {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let mut chan = self.channel.lock();
        let mut fut = Box::pin(chan.rpc_write(buf));
        Pin::new(&mut fut).poll(cx).map(|r| Ok(r))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

pub static mut STDOUT_CAP: usize = ProcessCSpace::Stdout as usize;

pub fn stdout() -> Stdout {
    static STDOUT: OnceCell<Arc<Mutex<RpcClient>>> = OnceCell::uninit();

    let inner = STDOUT
        .try_get_or_init(|| {
            let ep_server = EP_SERVER.try_get().unwrap();
            let (ntf_badge, ntf_ep) = ep_server.derive_badged_cap().unwrap();
            let client =
                RpcClient::connect(EpCap::new(unsafe { STDOUT_CAP }), ntf_ep, ntf_badge).unwrap();
            Arc::new(Mutex::new(client))
        })
        .unwrap()
        .clone();
    Stdout::new(inner)
}

pub struct Stdin {
    channel: Arc<Mutex<RpcClient>>,
}

impl Stdin {
    pub fn new(channel: Arc<Mutex<RpcClient>>) -> Self {
        Self { channel }
    }
}

impl AsyncRead for Stdin {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let mut chan = self.channel.lock();
        let mut fut = Box::pin(chan.rpc_read(buf));
        Pin::new(&mut fut).poll(cx).map(|r| Ok(r))
    }
}

impl Stream for Stdin {
    type Item = u8;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buf = [0u8; 1];
        self.poll_read(cx, &mut buf).map(|r| r.ok().map(|_| buf[0]))
    }
}

pub static mut STDIN_CAP: usize = ProcessCSpace::Stdout as usize;

pub fn stdin() -> Stdin {
    static STDIN: OnceCell<Arc<Mutex<RpcClient>>> = OnceCell::uninit();

    let inner = STDIN
        .try_get_or_init(|| {
            let ep_server = EP_SERVER.try_get().unwrap();
            let (ntf_badge, ntf_ep) = ep_server.derive_badged_cap().unwrap();
            let client =
                RpcClient::connect(EpCap::new(unsafe { STDOUT_CAP }), ntf_ep, ntf_badge).unwrap();
            Arc::new(Mutex::new(client))
        })
        .unwrap()
        .clone();
    Stdin::new(inner)
}

pub async fn _print(args: fmt::Arguments<'_>) {
    use futures_util::io::AsyncWriteExt;

    if let Err(e) = stdout().write_all(&format!("{}", args).into_bytes()).await {
        panic!("failed printing to stdout: {}", e);
    }
}
