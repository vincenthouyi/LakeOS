use core::sync::atomic::{AtomicPtr, AtomicBool, Ordering};
use core::mem::size_of;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use core::future::Future;

use alloc::sync::Arc;
use alloc::collections::VecDeque;

use futures_util::stream::Stream;

use spin::Mutex;

use rustyl4api::object::{EpCap, RamCap, RamObj};
use rustyl4api::ipc::IpcMessage;

use crate::space_manager::gsm;
use crate::io::{self, ErrorKind};
use crate::ep_server::{EpServer, EpMsgHandler};

const CACHELINE_SIZE: usize = 64;

struct MsgHdr {
    valid: AtomicBool,
    len: u8,
}

#[derive(Debug)]
struct ChannelState {
    write_sleep: AtomicBool,
    read_sleep: AtomicBool
}

const MSG_PAYLOAD_LEN: usize = CACHELINE_SIZE - size_of::<MsgHdr>();
const CHANNEL_SLOTS: usize = 4096 / 2 / size_of::<Msg>();
const CHANNEL_MSG_SLOTS: usize = CHANNEL_SLOTS - 1;
struct Msg {
    hdr: MsgHdr,
    payload: [u8; MSG_PAYLOAD_LEN],
}
const_assert_eq!(size_of::<Msg>(), CACHELINE_SIZE);

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
    buf_ptr: AtomicPtr<u8>,
    read_idx: usize,
    write_idx: usize,
}

impl UrpcStreamChannel {
    pub fn new(role: Role, ntf_ep: EpCap, buf_cap: RamCap, buf_ptr: *mut u8) -> Self {
        Self {
            role,
            ntf_ep,
            buf_cap,
            buf_ptr: AtomicPtr::new(buf_ptr),
            read_idx: 0,
            write_idx: 0,
        }
    }

    fn local_channel_state(&self) -> &ChannelState {
        let buf_base_ptr = self.buf_ptr.load(Ordering::Relaxed);
        let part = if self.role == Role::Server { 1 } else { 0 };

        unsafe {
            let channel_base_ptr = buf_base_ptr.offset(part * 2048);
            &*(channel_base_ptr as *const ChannelState)
        }
    }

    fn remote_channel_state(&self) -> &ChannelState {
        let buf_base_ptr = self.buf_ptr.load(Ordering::Relaxed);
        let part = if self.role == Role::Server { 0 } else { 1 };

        unsafe {
            let channel_base_ptr = buf_base_ptr.offset(part * 2048);
            &*(channel_base_ptr as *const ChannelState)
        }
    } 

    fn read_buffer(&self) -> &[Msg] {
        use core::slice::from_raw_parts;
        let buf_base_ptr = self.buf_ptr.load(Ordering::Relaxed);
        let part = if self.role == Role::Server { 0 } else { 1 };

        unsafe {
            from_raw_parts(
                (buf_base_ptr.offset(part * 2048) as *mut Msg).offset(1),
                CHANNEL_MSG_SLOTS,
            )
        }
    }

    fn write_buffer(&self) -> &mut [Msg] {
        use core::slice::from_raw_parts_mut;
        let buf_base_ptr = self.buf_ptr.load(Ordering::Relaxed);
        let part = if self.role == Role::Server { 1 } else { 0 };

        unsafe {
            from_raw_parts_mut(
                (buf_base_ptr.offset(part * 2048) as *mut Msg).offset(1),
                CHANNEL_MSG_SLOTS,
            )
        }
    }

    fn sleep_on_read(&self, x: bool) {
        self.local_channel_state().read_sleep.store(x, Ordering::SeqCst)
    }

    fn sleep_on_write(&self, x: bool) {
        self.local_channel_state().write_sleep.store(x, Ordering::SeqCst)
    }

    fn remote_sleep_on_read(&self) -> bool {
        self.remote_channel_state().read_sleep.load(Ordering::SeqCst)
    }

    fn remote_sleep_on_write(&self) -> bool {
        self.remote_channel_state().write_sleep.load(Ordering::SeqCst)
    }

    pub fn try_write_bytes(&mut self, buf: &[u8]) -> io::Result<usize> {
        let chan_buf = self.write_buffer();
        let mut write_idx = self.write_idx;
        let mut write_len = 0;

        for chunk in buf.chunks(MSG_PAYLOAD_LEN) {
            let chunk_len = chunk.len();
            let mut msg_ptr = &mut chan_buf[write_idx % CHANNEL_MSG_SLOTS];
            if msg_ptr.hdr.valid.load(Ordering::SeqCst) || chunk_len == 0 {
                break;
            }
            msg_ptr.payload[..chunk_len].copy_from_slice(chunk);
            msg_ptr.hdr.len = chunk_len as u8;
            write_len += chunk_len;
            write_idx += 1;
            msg_ptr.hdr.valid.store(true, Ordering::SeqCst);
        }

        if write_len == 0 {
            return Err(ErrorKind::WouldBlock)
        }

        self.write_idx = write_idx % CHANNEL_MSG_SLOTS;

        Ok(write_len)
    }

    pub fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    pub fn try_read_bytes(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let chan_buf = self.read_buffer();
        let mut read_idx = self.read_idx;
        let mut read_len = 0;
        let mut buf_rem_len = buf.len();

        while buf_rem_len > 0 {
            let msg_slot = &chan_buf[read_idx % CHANNEL_MSG_SLOTS];
            let msg_len = msg_slot.hdr.len as usize;
            if !msg_slot.hdr.valid.load(Ordering::SeqCst) {
                break;
            } else if buf_rem_len < msg_len {
                break;
            }

            buf[read_len..read_len + msg_len]
                .copy_from_slice(&msg_slot.payload[..msg_len]);

            read_len += msg_len;
            read_idx += 1;
            buf_rem_len -= msg_len;

            msg_slot.hdr.valid.store(false, Ordering::SeqCst);
        }

        if read_len == 0 {
            return Err(ErrorKind::WouldBlock)
        }

        self.read_idx = read_idx % CHANNEL_MSG_SLOTS;

        Ok(read_len)
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

    pub fn read_bytes(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut read_len = 0;

        while read_len < buf.len() {
            if let Some(b) = self.read_from_buffer() {
                buf[read_len] = b;
                read_len += 1;
            } else {
                let ret = self.inner.try_read_bytes(&mut self.buffer);
                match ret {
                    Ok(len) => {
                        self.buf_start = 0;
                        self.buf_end = len;
                    }
                    Err(ErrorKind::WouldBlock) => {
                        if self.inner.remote_sleep_on_write() {
                            self.inner.notify_remote_write();
                        }
                        break;
                    }
                    e => { return e }
                }
            }
        }
        if self.inner.remote_sleep_on_write() {
            self.inner.notify_remote_write();
        }

        Ok(read_len)
    }

    pub fn write_bytes(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut write_len = 0;

        while write_len < buf.len() {
            let ret = self.inner.try_write_bytes(buf);
            match ret {
                Ok(len) => { write_len += len }
                Err(ErrorKind::WouldBlock) => {
                    if self.inner.remote_sleep_on_read() {
                        self.inner.notify_remote_read();
                    }
                    continue
                }
                e => { return e }
            }
        }
        if self.inner.remote_sleep_on_read() {
            self.inner.notify_remote_read();
        }

        Ok(write_len)
    }
}

#[derive(Clone)]
pub struct UrpcStreamHandle (Arc<Mutex<UrpcStream>>);

impl UrpcStreamHandle {
    pub fn from_stream(stream: UrpcStream) -> Self {
        Self(Arc::new(Mutex::new(stream)))
    }

    pub fn poll_write<'a>(&self, buf: &'a[u8]) -> WriteFuture<'a> {
        WriteFuture::new(self.clone(), buf)
    }

    pub fn read_byte(&self) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.read_bytes(&mut buf)?;
        Ok(buf[0])
    }

    pub fn read_bytes(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.lock().read_bytes(buf)
    }

    pub fn write_bytes(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().write_bytes(buf)
    }
}

impl Stream for UrpcStreamHandle {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let reader = Pin::into_inner(self);
        match reader.read_byte() {
            Ok(byte) => Poll::Ready(Some(byte)),
            Err(_) => {
                reader.0.lock().inner.sleep_on_read(true);
                reader.0.lock().read_waker.lock().push_back(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl EpMsgHandler for UrpcStreamHandle {
    fn handle_ipc(&self, _ep_server: &EpServer, msg: IpcMessage, _cap_transfer_slot: Option<usize>) {
        if let IpcMessage::Message{payload, need_reply: _, cap_transfer: _, badge: _} = msg {
            let direction = payload[0];
            let inner = self.0.lock();
            if direction == 0 {
                let mut read_waker = inner.read_waker.lock();
                while let Some(waker) = read_waker.pop_front() {
                    waker.wake();
                }
                inner.inner.sleep_on_read(false);
            } else if direction == 1 {
                let mut write_waker = inner.write_waker.lock();
                while let Some(waker) = write_waker.pop_front() {
                    waker.wake();
                }
                inner.inner.sleep_on_write(false);
            }
        }
    }
}

pub struct WriteFuture<'a> {
    inner: UrpcStreamHandle,
    buf: &'a [u8],
    write_len: usize,
}

impl<'a> WriteFuture<'a> {
    pub fn new(urpc: UrpcStreamHandle, buf: &'a [u8]) -> Self {
        Self { inner: urpc, buf: buf, write_len : 0 }
    }
}

impl<'a> Future for WriteFuture<'a> {
    type Output = io::Result<usize>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let inner = Pin::into_inner(self);
        while inner.write_len < inner.buf.len() {
            let mut stream = inner.inner.0.lock();
            let ret = stream.inner.try_write_bytes(&inner.buf[inner.write_len..]);
            match ret {
                Ok(write_len) => { inner.write_len += write_len }
                Err(ErrorKind::WouldBlock) => {
                    stream.inner.sleep_on_write(true);
                    stream.write_waker.lock().push_back(cx.waker().clone());
                    return Poll::Pending;
                }
                e => { return Poll::Ready(e) }
            }
        }
        Poll::Ready(Ok(inner.write_len))
    }
}