use core::fmt;
use core::task::{Poll, Context};
use core::pin::Pin;
use core::future::Future;

use alloc::{
    sync::Arc,
    format,
};

use spin::Mutex;

use conquer_once::spin::OnceCell;

use futures_util::io::{AsyncWrite, AsyncRead};
use futures_util::stream::Stream;

use rustyl4api::object::EpCap;
use rustyl4api::process::ProcessCSpace;

use crate::io;
use crate::ep_server::{EP_SERVER};
use crate::rpc::{RpcClient, RpcCallFuture};

pub struct Stdout {
    channel: Arc<Mutex<RpcClient>>,
    rpc_state: Option<RpcCallFuture>,
}

impl Stdout {
    pub fn new(channel: Arc<Mutex<RpcClient>>) -> Self {
        Self {
            channel: channel,
            rpc_state: None,
        }
    }
}

impl AsyncWrite for Stdout
{
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8])
        -> Poll<io::Result<usize>>
    {
        let Self { channel, rpc_state} = &mut *self;
        let s = rpc_state.get_or_insert(channel.lock().rpc_write(buf));
        Pin::new(s).poll(cx)
            .map(|_| {
                self.rpc_state.take();
                Ok(buf.len())
            })
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

pub static mut STDOUT_CAP: usize = ProcessCSpace::Stdout as usize;

pub fn stdout() -> Stdout {
    static STDOUT: OnceCell<Arc<Mutex<RpcClient>>> = OnceCell::uninit(); 

    let inner = STDOUT.try_get_or_init(|| {
        let ep_server = EP_SERVER.try_get().unwrap();
        let (ntf_badge, ntf_ep) = ep_server.derive_badged_cap().unwrap();
        let client = RpcClient::connect(EpCap::new(unsafe { STDOUT_CAP }), ntf_ep, ntf_badge).unwrap();
        Arc::new(Mutex::new(client))
    }).unwrap().clone();
    Stdout::new(inner)
}

pub struct Stdin {
    channel: Arc<Mutex<RpcClient>>,
    rpc_state: Option<RpcCallFuture>,
}

impl Stdin {
    pub fn new(channel: Arc<Mutex<RpcClient>>) -> Self {
        Self {
            channel: channel,
            rpc_state: None,
        }
    }
}

impl AsyncRead for Stdin {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8])
        -> Poll<io::Result<usize>>
    {
        let Self { channel, rpc_state } = &mut *self;
        let s = rpc_state.get_or_insert(channel.lock().rpc_read(buf));
        Pin::new(s).poll(cx)
            .map(|resp| {
                self.rpc_state.take();
                let resp : crate::rpc::ReadResponse = serde_json::from_slice(&resp.msg).unwrap();
                buf[..resp.buf.len()].copy_from_slice(&resp.buf);
                Ok(resp.buf.len())
            })
    }
}

impl Stream for Stdin {
    type Item = u8;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_> )
        -> Poll<Option<Self::Item>>
    {
        let mut buf = [0u8; 1];
        self.poll_read(cx, &mut buf)
            .map(|r| {
                r.ok().map(|_| buf[0])
            })
    }
}

pub static mut STDIN_CAP: usize = ProcessCSpace::Stdout as usize;

pub fn stdin() -> Stdin {
    static STDIN: OnceCell<Arc<Mutex<RpcClient>>> = OnceCell::uninit(); 

    let inner = STDIN.try_get_or_init(|| {
        let ep_server = EP_SERVER.try_get().unwrap();
        let (ntf_badge, ntf_ep) = ep_server.derive_badged_cap().unwrap();
        let client = RpcClient::connect(EpCap::new(unsafe { STDOUT_CAP }), ntf_ep, ntf_badge).unwrap();
        Arc::new(Mutex::new(client))
    }).unwrap().clone();
    Stdin::new(inner)
}

pub async fn _print(args: fmt::Arguments<'_>) {
    use futures_util::io::AsyncWriteExt;

    if let Err(e) = stdout().write_all(&format!("{}", args).into_bytes()).await {
        panic!("failed printing to stdout: {}", e);
    }
}