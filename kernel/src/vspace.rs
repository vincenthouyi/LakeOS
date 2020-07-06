pub use crate::arch::vspace::{VSpace, Table, Entry, Shareability, AccessPermission, MemoryAttr, PAGE_SIZE};

use crate::prelude::*;

#[derive(Debug)]
pub enum VSpaceError {
    L2EntryNotFound,
    L3EntryNotFound,
    L4EntryNotFound,
    DeleteFirst,
    Other,
}

impl core::convert::From<VSpaceError> for sysapi::error::SysError {
    fn from(_e: VSpaceError) -> Self {
        //TODO: add more specific SysError subtypes for vspace error
        Self::VSpaceError
    }
}

pub type VSpaceResult<T> = core::result::Result<T, VSpaceError>;
