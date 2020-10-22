use core::mem::size_of;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use core::sync::atomic::{fence, Ordering};

use alloc::sync::Arc;
use alloc::collections::VecDeque;

use volatile::Volatile;
use spin::Mutex;

use rustyl4api::object::{EpCap, RamCap, RamObj};
use rustyl4api::ipc::IpcMessage;

use crate::space_manager::gsm;
use crate::io::{self, AsyncRead, AsyncWrite};
use crate::stream::Stream;
use crate::ep_server::{EpServer, EpMsgHandler};

const CACHELINE_SIZE: usize = 64;

#[repr(C)]
struct MsgHdr {
    valid: u8,
    len: u8,
    padding: [u8; 6],
}

struct ChannelState {
    write_sleep: bool,
    read_sleep: bool 
}

const MSG_PAYLOAD_LEN: usize = CACHELINE_SIZE - size_of::<MsgHdr>();
const CHANNEL_SLOTS: usize = 4096 / 2 / size_of::<MsgSlot>();
const CHANNEL_MSG_SLOTS: usize = CHANNEL_SLOTS - 1;
struct MsgSlot {
    hdr: MsgHdr,
    payload: [u8; MSG_PAYLOAD_LEN],
}
const_assert_eq!(size_of::<MsgSlot>(), CACHELINE_SIZE);

#[derive(Debug, PartialEq, Eq)]
pub enum Role {
    Server,
    Client,
}

#[derive(Debug)]
pub struct UrpcStreamChannel {
    role: Role,
    ntf_ep: EpCap,
    buf_cap: RamCap,
    buf_ptr: usize,
    read_idx: usize,
    write_idx: usize,
}

impl UrpcStreamChannel {
    pub fn new(role: Role, ntf_ep: EpCap, buf_cap: RamCap, buf_ptr: *mut u8) -> Self {
        Self {
            role,
            ntf_ep,
            buf_cap,
            buf_ptr: buf_ptr as usize,
            read_idx: 0,
            write_idx: 0,
        }
    }

    fn local_channel_state(&self) -> &mut ChannelState {
        let buf_base_ptr = self.buf_ptr as *mut u8;
        let part = if self.role == Role::Server { 1 } else { 0 };

        unsafe {
            let channel_base_ptr = buf_base_ptr.offset(part * 2048);
            &mut *(channel_base_ptr as *mut ChannelState)
        }
    }

    fn remote_channel_state(&self) -> &ChannelState {
        let buf_base_ptr = self.buf_ptr as *mut u8;
        let part = if self.role == Role::Server { 0 } else { 1 };

        unsafe {
            let channel_base_ptr = buf_base_ptr.offset(part * 2048);
            &*(channel_base_ptr as *const ChannelState)
        }
    } 

    fn read_buffer(&self) -> &mut [MsgSlot] {
        use core::slice::from_raw_parts_mut;
        let buf_base_ptr = self.buf_ptr as *mut u8;
        let part = if self.role == Role::Server { 0 } else { 1 };

        unsafe {
            from_raw_parts_mut(
                (buf_base_ptr.offset(part * 2048) as *mut MsgSlot).offset(1),
                CHANNEL_MSG_SLOTS,
            )
        }
    }

    fn write_buffer(&self) -> &mut [MsgSlot] {
        use core::slice::from_raw_parts_mut;
        let buf_base_ptr = self.buf_ptr as *mut u8;
        let part = if self.role == Role::Server { 1 } else { 0 };

        unsafe {
            from_raw_parts_mut(
                (buf_base_ptr.offset(part * 2048) as *mut MsgSlot).offset(1),
                CHANNEL_MSG_SLOTS,
            )
        }
    }

    fn sleep_on_read(&mut self, x: bool) {
        Volatile::new(&mut self.local_channel_state().read_sleep).write(x)
    }

    fn sleep_on_write(&mut self, x: bool) {
        Volatile::new(&mut self.local_channel_state().write_sleep).write(x)
    }

    fn remote_sleep_on_read(&self) -> bool {
        Volatile::new(&self.remote_channel_state().read_sleep).read()
    }

    fn remote_sleep_on_write(&self) -> bool {
        Volatile::new(&self.remote_channel_state().write_sleep).read()
    }

    pub fn write_slot(&mut self, buf: &[u8]) -> io::Result<usize> {
        let chan_buf = self.write_buffer();
        let write_idx = self.write_idx;

        let msg_slot = &mut chan_buf[write_idx % CHANNEL_MSG_SLOTS];
        fence(Ordering::SeqCst);
        let mut valid = Volatile::new(&mut msg_slot.hdr.valid);
        if valid.read() == 1 {
            fence(Ordering::SeqCst);
            return Err(io::ErrorKind::WouldBlock.into());
        }
        fence(Ordering::SeqCst);

        let msglen = buf.len().min(MSG_PAYLOAD_LEN);
        msg_slot.payload[..msglen].copy_from_slice(&buf[..msglen]);
        msg_slot.hdr.len = msglen as u8;
        fence(Ordering::SeqCst);
        valid.write(1);

        self.write_idx = (write_idx + 1) % CHANNEL_MSG_SLOTS;

        Ok(msglen)
    }

    pub fn read_slot(&mut self, buf: &mut [u8; MSG_PAYLOAD_LEN]) -> io::Result<usize> {
        let chan_buf = self.read_buffer();
        let read_idx = self.read_idx;

        let msg_slot = &mut chan_buf[read_idx % CHANNEL_MSG_SLOTS];
        let mut valid = Volatile::new(&mut msg_slot.hdr.valid);

        fence(Ordering::SeqCst);
        if valid.read() != 1 {
            fence(Ordering::SeqCst);
            return Err(io::ErrorKind::WouldBlock.into())
        }
        fence(Ordering::SeqCst);

        let msg_len = msg_slot.hdr.len as usize;
        buf[..msg_len].copy_from_slice(&msg_slot.payload[..msg_len]);

        fence(Ordering::SeqCst);
        valid.write(0);

        self.read_idx = (read_idx + 1) % CHANNEL_MSG_SLOTS;

        Ok(msg_len)
    }

    pub fn notify_remote_write(&self) {
        self.ntf_ep.send(&[1], None).unwrap();
    }

    pub fn notify_remote_read(&self) {
        self.ntf_ep.send(&[0], None).unwrap();
    }
}

pub struct UrpcStream {
    inner: UrpcStreamChannel,
    read_waker: Arc<Mutex<VecDeque<Waker>>>,
    write_waker: Arc<Mutex<VecDeque<Waker>>>,
    buffer: [u8; MSG_PAYLOAD_LEN],
    buf_start: usize,
    buf_end: usize,
}

impl UrpcStream {
    pub fn connect(ep: EpCap, ntf_ep: EpCap, _ntf_badge: usize) -> io::Result<Self> {
        use rustyl4api::vspace::Permission;
        use rustyl4api::object::ReplyCap;

        /* Connect by sending client notification ep */
        let trans_cap_slot = ntf_ep.slot;
        let _ret = ep.call(&[], Some(trans_cap_slot)).unwrap();
        let svr_ntf_ep = EpCap::new(trans_cap_slot);

        /* Generate buffer cap and map to current VSpace */
        let buf_cap = gsm!().alloc_object::<RamObj>(12).unwrap();
        let buf_ptr = gsm!().insert_ram_at(buf_cap.clone(), 0, Permission::writable());

        /* Derive a copy of buffer cap and send to server */
        let copied_buf_cap_slot = gsm!().cspace_alloc().unwrap();
        buf_cap.derive(copied_buf_cap_slot).unwrap();
        let reply_cap = ReplyCap::new(0);
        reply_cap.reply(&[], Some(copied_buf_cap_slot)).unwrap();

        let channel = UrpcStreamChannel::new(Role::Client, svr_ntf_ep, buf_cap, buf_ptr);

        Ok(Self::from_stream(channel))
    }

    pub fn from_stream(stream: UrpcStreamChannel) -> Self {
        Self {
            inner: stream,
            read_waker: Arc::new(Mutex::new(VecDeque::new())),
            write_waker: Arc::new(Mutex::new(VecDeque::new())),
            buffer: [0u8; MSG_PAYLOAD_LEN],
            buf_start: 0,
            buf_end: 0,
        }
    }

    fn buf_data_len(&self) -> usize {
        self.buf_end - self.buf_start
    }

    fn read_from_buffer(&mut self) -> Option<u8> {
        if self.buf_data_len() != 0 {
            let byte = self.buffer[self.buf_start];
            self.buf_start += 1;
            return Some(byte);
        }

        None
    }

    fn refill_buffer(&mut self) -> io::Result<usize> {
        let len = self.inner.read_slot(&mut self.buffer)?;
        self.buf_start = 0;
        self.buf_end = len;
        return Ok(len);
    }

    pub fn read_bytes(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut read_len = 0;

        while read_len < buf.len() {
            if let Some(b) = self.read_from_buffer() {
                buf[read_len] = b;
                read_len += 1;
            } else {
                match self.refill_buffer() {
                    Ok(_) => { }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => { break }
                    e => { return e }
                }
            }
        }
        if self.inner.remote_sleep_on_write() {
            self.inner.notify_remote_write();
        }

        if read_len == 0 {
            Err(io::ErrorKind::WouldBlock.into())
        } else {
            Ok(read_len)
        }
    }

    pub fn write_bytes(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut write_len = 0;

        while write_len < buf.len() {
            match self.inner.write_slot(&buf[write_len..]) {
                Ok(len) => { write_len += len }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => { break }
                e => { return e }
            }
        }
        if self.inner.remote_sleep_on_read() {
            self.inner.notify_remote_read();
        }

        Ok(write_len)
    }
}

impl AsyncRead for UrpcStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8]
    ) -> Poll<Result<usize, io::Error>> {
        let reader = Pin::into_inner(self);
        match reader.read_bytes(buf) {
            Ok(n) => Poll::Ready(Ok(n)),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                reader.inner.sleep_on_read(true);
                reader.read_waker.lock().push_back(cx.waker().clone());
                Poll::Pending
            }
            e => { Poll::Ready(e) }
        }
    }
}

impl AsyncWrite for UrpcStream
{
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8])
        -> Poll<io::Result<usize>>
    {
        let inner = Pin::into_inner(self);
        match inner.write_bytes(buf) {
            Ok(n) => Poll::Ready(Ok(n)),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                inner.inner.sleep_on_write(true);
                inner.write_waker.lock().push_back(cx.waker().clone());
                Poll::Pending
            },
            e => { Poll::Ready(e) }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[derive(Clone)]
pub struct UrpcStreamHandle (Arc<Mutex<UrpcStream>>);

impl UrpcStreamHandle {
    pub fn from_stream(stream: UrpcStream) -> Self {
        Self(Arc::new(Mutex::new(stream)))
    }

    // Blocking read
    pub fn read_bytes(&self, buf: &mut [u8]) -> io::Result<usize> {
        let mut readlen = 0;
        let mut guard = self.0.lock();
        while readlen == 0 {
            match guard.read_bytes(&mut buf[readlen..]) {
                Ok(len) => { readlen += len }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => { continue }
                Err(e) => { return Err(e) }
            }
        }
        Ok(readlen)
    }

    pub fn write_bytes(&self, buf: &[u8]) -> io::Result<usize> {
        let mut writelen = 0;
        let mut guard = self.0.lock();

        while writelen < buf.len() {
            match guard.write_bytes(&buf[writelen..]) {
                Ok(len) => { writelen += len }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => { continue }
                Err(e) => { return Err(e) }
            }
        }
        Ok(writelen)
    }
}

impl Stream for UrpcStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let mut buf = [0; 1];
        self.poll_read(cx, &mut buf)
            .map(|e| e.ok().map(|_| buf[0]))
    }
}

impl EpMsgHandler for UrpcStreamHandle {
    fn handle_ipc(&self, _ep_server: &EpServer, msg: IpcMessage, _cap_transfer_slot: Option<usize>) {
        if let IpcMessage::Message{payload, need_reply: _, cap_transfer: _, badge: _} = msg {
            let direction = payload[0];
            let mut inner = self.0.lock();
            if direction == 0 {
                while let Some(waker) = inner.read_waker.lock().pop_front() {
                    waker.wake();
                }
                inner.inner.sleep_on_read(false);
            } else if direction == 1 {
                while let Some(waker) = inner.write_waker.lock().pop_front() {
                    waker.wake();
                }
                inner.inner.sleep_on_write(false);
            }
        }
    }
}

impl AsyncRead for UrpcStreamHandle {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8]
    ) -> Poll<Result<usize, io::Error>> {
        Pin::new(&mut *(&*self).0.lock()).poll_read(cx, buf)
    }
}

impl AsyncWrite for UrpcStreamHandle
{
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8])
        -> Poll<io::Result<usize>>
    {
        Pin::new(&mut *(&*self).0.lock()).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut *(&*self).0.lock()).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut *(&*self).0.lock()).poll_close(cx)
    }
}

impl Stream for UrpcStreamHandle {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        Pin::new(&mut *(&*self).0.lock()).poll_next(cx)
    }
}