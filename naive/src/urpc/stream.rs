use core::sync::atomic::{AtomicPtr, AtomicBool, AtomicUsize, Ordering};
use core::mem::size_of;

use crate::space_manager::gsm;
use crate::io;

use rustyl4api::object::{EpCap, RamCap, RamObj, ReplyCap};

const CACHELINE_SIZE: usize = 64;

struct MsgHdr {
    valid: AtomicBool,
    len: u8,
}

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
pub struct UrpcStream {
    role: Role,
    ntf_ep: EpCap,
    buf_cap: RamCap,
    buf_ptr: AtomicPtr<u8>,
    read_idx: AtomicUsize,
    write_idx: AtomicUsize,
}

impl UrpcStream {
    pub fn new(role: Role, ntf_ep: EpCap, buf_cap: RamCap, buf_ptr: *mut u8) -> Self {
        Self {
            role,
            ntf_ep,
            buf_cap,
            buf_ptr: AtomicPtr::new(buf_ptr),
            read_idx: AtomicUsize::new(0),
            write_idx: AtomicUsize::new(0),
        }
    }

    pub fn connect(ep: EpCap, ntf_ep: EpCap, ntf_badge: usize) -> io::Result<Self> {
        use rustyl4api::vspace::Permission;

        /* Connect by sending client notification ep */
        let trans_cap_slot = ntf_ep.slot;
        let ret = ep.call(&[], Some(trans_cap_slot)).unwrap();
        let svr_ntf_ep = EpCap::new(trans_cap_slot);

        /* Generate buffer cap and map to current VSpace */
        let buf_cap = gsm!().alloc_object::<RamObj>(12).unwrap();
        let buf_ptr = gsm!().insert_ram_at(buf_cap.clone(), 0, Permission::writable());

        /* Derive a copy of buffer cap and send to server */
        let copied_buf_cap_slot = gsm!().cspace_alloc().unwrap();
        buf_cap.derive(copied_buf_cap_slot).unwrap();
        let reply = ReplyCap::new(0);
        reply.reply(&[], Some(copied_buf_cap_slot)).unwrap();

        Ok(Self::new(Role::Client, svr_ntf_ep, buf_cap, buf_ptr))
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

    pub fn sleep_on_read(&self) {
        self.local_channel_state().read_sleep.store(true, Ordering::SeqCst)
    }

    pub fn sleep_on_write(&self) {
        self.local_channel_state().write_sleep.store(true, Ordering::SeqCst)
    }

    pub fn try_write_bytes(&self, buf: &[u8]) -> io::Result<usize> {
        let chan_buf = self.write_buffer();
        let mut write_idx = self.write_idx.load(Ordering::Relaxed);
        let mut write_len = 0;

        for chunk in buf.chunks(MSG_PAYLOAD_LEN) {
            let chunk_len = chunk.len();
            let mut msg_ptr = &mut chan_buf[write_idx % CHANNEL_MSG_SLOTS];
            if msg_ptr.hdr.valid.load(Ordering::SeqCst) || chunk_len == 0 {
                break;
            }
            msg_ptr.payload[..chunk_len].copy_from_slice(chunk);
            msg_ptr.hdr.len = chunk_len as u8;
            msg_ptr.hdr.valid.store(true, Ordering::SeqCst);
            write_len += chunk_len;
            write_idx += 1;

            msg_ptr.hdr.valid.store(true, Ordering::SeqCst);
        }

        self.write_idx.store(write_idx % CHANNEL_MSG_SLOTS, Ordering::Relaxed);
        if self.remote_channel_state().read_sleep.load(Ordering::SeqCst) {
            self.ntf_ep.send(&[0, write_len], None).unwrap();
        }

        Ok(write_len)
    }

    pub fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    pub fn try_read_bytes(&self, buf: &mut [u8]) -> io::Result<usize> {
        let chan_buf = self.read_buffer();
        let mut read_idx = self.read_idx.load(Ordering::Relaxed);
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

        self.read_idx.store(read_idx % CHANNEL_MSG_SLOTS, Ordering::Relaxed);
        if self.remote_channel_state().write_sleep.load(Ordering::SeqCst) {
            self.ntf_ep.send(&[1, read_len], None).unwrap();
        }

        Ok(read_len)
    }

    pub fn read_bytes(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut read_len = 0;

        while read_len == 0 {
            read_len = self.try_read_bytes(buf)?;
        }

        Ok(read_len)
    }

    pub fn write_bytes(&self, buf: &[u8]) -> io::Result<usize> {
        let mut write_len = 0;

        while write_len == 0 {
            write_len = self.try_write_bytes(buf)?;
        }

        Ok(write_len)
    }
}