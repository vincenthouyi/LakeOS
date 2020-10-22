use crate::io::AsyncWrite;
use futures_core::future::Future;
use futures_core::task::{Context, Poll};
use bare_io as io;
use core::pin::Pin;
use core::fmt::Arguments;

/// Future for the [`write`](super::AsyncWriteExt::write) method.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct WriteFmt<'a, W: ?Sized> {
    writer: &'a mut W,
    args: Arguments<'a>,
}

impl<W: ?Sized + Unpin> Unpin for WriteFmt<'_, W> {}

impl<'a, W: AsyncWrite + ?Sized + Unpin> WriteFmt<'a, W> {
    pub(super) fn new(writer: &'a mut W, args: Arguments<'a>) -> Self {
        Self { writer, args }
    }
}

impl<W: AsyncWrite + ?Sized + Unpin> Future for WriteFmt<'_, W> {
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use crate::alloc::string::ToString;

        let buf = &self.args.to_string().into_bytes();
        let this = &mut *self;
        Pin::new(&mut this.writer).poll_write(cx, buf)
    }
}