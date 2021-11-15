use log::trace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RamType {
    Invalid = 0,
    // KernelBinary,
    // KernelStack,
    KernelPageTable,
    KernelPage,
    BootLoader,
    InitRamFS,
    FreeSpace,
}

#[derive(Debug, Clone, Copy)]
pub struct RamInfo {
    pub base: usize,
    pub size: usize,
    pub mem_type: RamType,
}

#[derive(Debug, Clone, Copy)]
pub enum BootInfoEntry {
    NullEntry,
    RamEntry(RamInfo),
}

#[derive(Debug)]
pub struct BootInfo {
    pub entries: &'static mut [BootInfoEntry],
    idx: usize,
}

impl BootInfo {
    pub unsafe fn new_from_frame(frame_base: *mut u8, init: bool) -> Self {
        use core::mem::size_of;
        use core::slice::from_raw_parts_mut;

        let entries = 4096 / size_of::<BootInfoEntry>();
        let ret = Self {
            entries: from_raw_parts_mut(frame_base as *mut BootInfoEntry, entries),
            idx: 0,
        };

        if init {
            for ent in ret.entries.iter_mut() {
                *ent = BootInfoEntry::NullEntry;
            }
        }

        return ret;
    }

    pub fn append_entry(&mut self, entry: BootInfoEntry) {
        trace!("inserting entry[{}] {:?}", self.idx, entry);
        if self.idx >= self.entries.len() {
            return;
        }
        self.entries[self.idx] = entry;
        self.idx += 1
    }
}
