use crate::page_table_entry::PageTableEntry;

pub trait Level {
    const LEVEL: usize;
}

pub trait TableLevel: Level {
    type NextLevel: Level;
    type EntryType: PageTableEntry;
    const TABLE_ENTRIES: usize;
}

pub trait TopLevel: TableLevel {}

pub trait PageLevel: Level {
    const FRAME_BIT_SIZE: usize;
}
