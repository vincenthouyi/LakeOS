use core::{fmt, str, slice, iter};

const HEADER_LEN: usize = 110; // length of fixed part
const MAGIC: &[u8] = b"070701";
const TRAILER_NAME: &str = "TRAILER!!!";

#[repr(C)]
pub struct Entry {
    magic: [u8; 6],
    ino: [u8; 8],
    mode: [u8; 8],
    uid: [u8; 8],
    gid: [u8; 8],
    nlink: [u8; 8],
    mtime: [u8; 8],
    file_size: [u8; 8],
    dev_major: [u8; 8],
    dev_minor: [u8; 8],
    rdev_major: [u8; 8],
    rdev_minor: [u8; 8],
    name_len: [u8; 8],
    check_sum: [u8; 8],
    name: [u8]
}

fn newc_atoi(b: &[u8; 8]) -> u32 {
    let s = str::from_utf8(b).unwrap();
    u32::from_str_radix(s, 16).unwrap()
}

impl Entry {
    pub fn inode(&self) -> u32 {
        newc_atoi(&self.ino)
    }
    pub fn mode(&self) -> u32 {
        newc_atoi(&self.mode)
    }
    pub fn uid(&self) -> u32 {
        newc_atoi(&self.uid)
    }
    pub fn gid(&self) -> u32 {
        newc_atoi(&self.gid)
    }
    pub fn nlink(&self) -> u32 {
        newc_atoi(&self.nlink)
    }
    pub fn mtime(&self) -> u32 {
        newc_atoi(&self.mtime)
    }
    pub fn file_size(&self) -> u32 {
        newc_atoi(&self.file_size)
    }
    pub fn dev_major(&self) -> u32 {
        newc_atoi(&self.dev_major)
    }
    pub fn dev_minor(&self) -> u32 {
        newc_atoi(&self.dev_minor)
    }
    pub fn rdev_major(&self) -> u32 {
        newc_atoi(&self.rdev_major)
    }
    pub fn rdev_minor(&self) -> u32 {
        newc_atoi(&self.rdev_minor)
    }
    pub fn name_len(&self) -> u32 {
        newc_atoi(&self.name_len)
    }
    pub fn check_sum(&self) -> u32 {
        newc_atoi(&self.check_sum)
    }
    pub fn name(&self) -> &str {
        let len = self.name_len() as usize;
        str::from_utf8(&self.name[..len - 1]).unwrap()
    }
    pub fn header_size(&self) -> usize {
        align_up(HEADER_LEN + self.name_len() as usize, 4)
    }
    pub fn total_size(&self) -> usize {
        self.header_size() + self.file_size() as usize
    }
    pub fn entry_size(&self) -> usize {
        align_up(self.total_size(), 4)
    }
    pub fn content(&self) -> &[u8] {
        let buf = unsafe {
            let ptr = self as *const _ as *const u8;
            slice::from_raw_parts(ptr, self.total_size())
        };
        &buf[self.header_size()..self.total_size()]
    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("Entry")
            .field("inode", &self.inode())
            .field("mode", &self.mode())
            .field("uid", &self.uid())
            .field("gid", &self.gid())
            .field("nlink", &self.nlink())
            .field("mtime", &self.mtime())
            .field("file_size", &self.file_size())
            .field("dev_major", &self.dev_major())
            .field("dev_minor", &self.dev_minor())
            .field("rdev_major", &self.rdev_major())
            .field("rdev_minor", &self.rdev_minor())
            .field("name_len", &self.name_len())
            .field("check_sum", &self.check_sum())
            .field("name", &self.name())
            .finish()
    }
}

impl Entry {
    pub fn from_bytes(bytes: &[u8]) -> Option<&Self> {
        let entry = unsafe {
            &*(bytes as *const _ as *const Entry)
        };
        if entry.magic != MAGIC ||
            entry.total_size() > bytes.len() ||
            entry.name() == TRAILER_NAME
        {
            return None;
        }

        Some(entry)
    }
}

pub struct Reader<'a> {
    inner: &'a [u8]
}

impl<'a> Reader<'a> {
    pub fn from_bytes(inner: &'a [u8]) -> Self {
        Self { inner }
    }

    pub fn entries(&self) -> EntryIter<'a> {
        EntryIter {
            inner: self.inner,
            offset: 0
        }
    }
}

pub struct EntryIter<'a> {
    inner: &'a [u8],
    offset: usize,
}

impl<'a> iter::Iterator for EntryIter<'a> {
    type Item = &'a Entry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset > self.inner.len() {
            return None
        }

        let entry = Entry::from_bytes(&self.inner[self.offset..]);

        if let Some(e) = entry {
            self.offset += e.entry_size()
        }
        entry
    }
}

pub const fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

pub const fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr.saturating_add(align - 1), align)
}