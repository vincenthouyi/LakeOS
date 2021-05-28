use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{
    convert::AsRef,
    pin::Pin,
    task::{Context, Poll},
};

use spin::Mutex;

use futures_util::future::BoxFuture;
use futures_util::io::{AsyncRead, AsyncWrite};
use futures_util::ready;

use crate::{
    ep_server::EP_SERVER,
    io,
    ns::ns_client,
    objects::EpCap,
    path::{Path, PathBuf},
    rpc::RpcClient,
    Result,
    Error,
};

pub struct File {
    client: Arc<Mutex<RpcClient>>,
    offset: usize,
    read_state: Option<BoxFuture<'static, Result<Vec<u8>>>>,
    write_state: Option<BoxFuture<'static, Result<usize>>>,
}

impl File {
    pub fn new(client: RpcClient) -> Self {
        Self {
            client: Arc::new(Mutex::new(client)),
            offset: 0,
            read_state: None,
            write_state: None,
        }
    }

    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = super::canonicalize(path)?;
        let resp_cap = ns_client().await.lock().lookup_service(&path).await?;
        let ret = Self::connect(&resp_cap).await;
        ret
    }

    pub async fn connect(ep: &EpCap) -> Result<Self> {
        let receiver = EP_SERVER.derive_receiver().ok_or(Error::NoReceiver)?;
        let cli = RpcClient::connect(ep, receiver).await?;
        Ok(Self::new(cli))
    }

    pub async fn read_dir(&mut self) -> Result<Vec<PathBuf>> {
        self.client.lock().read_dir().await
    }
}

impl AsyncWrite for File {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let Self {
            client,
            offset,
            read_state: _,
            write_state,
        } = &mut *self;
        let write_fut = write_state.get_or_insert_with(|| {
            let fut_cli = client.clone();
            let buf = buf.to_vec();
            let fut = || async move {
                let fut_cli = fut_cli;
                let mut cli_guard = fut_cli.lock();
                cli_guard.rpc_write(buf).await
            };
            Box::pin(fut())
        });

        let write_len = ready!(write_fut.as_mut().poll(cx))
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "foo"))?;

        *offset += write_len;
        write_state.take();
        Poll::Ready(Ok(write_len))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

impl AsyncRead for File {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let Self {
            client,
            offset,
            read_state,
            write_state: _,
        } = &mut *self;
        let buflen = buf.len();
        let read_fut = read_state.get_or_insert_with(|| {
            let fut_cli = client.clone();
            let buflen = buflen;
            let offset = *offset;
            let fut = || async move {
                let fut_cli = fut_cli;
                let mut cli_guard = fut_cli.lock();
                cli_guard.rpc_read(buflen, offset).await
            };
            Box::pin(fut())
        });

        let read_buf = ready!(read_fut.as_mut().poll(cx))
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "foo"))?;

        let read_len = read_buf.len();
        *offset += read_len;
        read_state.take();
        let copy_len = buf.len().min(read_buf.len());
        buf[..copy_len].copy_from_slice(&read_buf[..copy_len]);
        Poll::Ready(Ok(read_len))
    }
}
