
pub enum Error {
    TableMiss { level: usize },
    SlotOccupied { level: usize },
    SlotEmpty,
}

pub type Result<T> = core::result::Result<T, Error>;