#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub enum SysError {
    OK,
    CSpaceNotFound,
    CapabilityTypeError,
    LookupError,
    UnableToDerive,
    SlotNotEmpty,
    SlotEmpty,
    UnsupportedSyscallOp,
    VSpaceCapMapped,
    VSpaceCapNotMapped,
    VSpaceTableMiss { level: u8 },
    VSpaceSlotOccupied { level: u8 },
    VSpacePermissionError,
    InvalidValue,

    /* Untyped */
    SizeTooSmall,
}

impl SysError {
    pub const fn errno(&self) -> SysErrno {
        match self {
            SysError::OK => SysErrno::OK,
            SysError::CSpaceNotFound => SysErrno::CSpaceNotFound,
            SysError::CapabilityTypeError => SysErrno::CapabilityTypeError,
            SysError::LookupError => SysErrno::LookupError,
            SysError::UnableToDerive => SysErrno::UnableToDerive,
            SysError::SlotNotEmpty => SysErrno::SlotNotEmpty,
            SysError::SlotEmpty => SysErrno::SlotEmpty,
            SysError::UnsupportedSyscallOp => SysErrno::UnsupportedSyscallOp,
            SysError::VSpaceCapMapped => SysErrno::VSpaceCapMapped,
            SysError::VSpaceCapNotMapped => SysErrno::VSpaceCapNotMapped,
            SysError::VSpaceTableMiss { level: _ } => SysErrno::VSpaceTableMiss,
            SysError::VSpaceSlotOccupied { level: _ } => SysErrno::VSpaceSlotOccupied,
            SysError::VSpacePermissionError => SysErrno::VSpacePermissionError,
            SysError::InvalidValue => SysErrno::InvalidValue,
            SysError::SizeTooSmall => SysErrno::SizeTooSmall,
        }
    }
}

#[derive(Debug, Copy, Clone, ToPrimitive, FromPrimitive, Eq, PartialEq)]
#[repr(C)]
pub enum SysErrno {
    OK = 0,
    CSpaceNotFound,
    CapabilityTypeError,
    LookupError,
    UnableToDerive,
    SlotNotEmpty,
    SlotEmpty,
    UnsupportedSyscallOp,
    VSpaceCapMapped,
    VSpaceCapNotMapped,
    VSpaceTableMiss,
    VSpaceSlotOccupied,
    VSpacePermissionError,
    InvalidValue,

    /* Untyped */
    SizeTooSmall,
}

pub type SysResult<T> = core::result::Result<T, SysError>;

impl core::convert::Into<SysErrno> for SysError {
    fn into(self) -> SysErrno {
        self.errno()
    }
}

impl From<vspace::Error> for SysError {
    fn from(e: vspace::Error) -> SysError {
        match e {
            vspace::Error::TableMiss { level } => SysError::VSpaceTableMiss { level: level as u8 },
            vspace::Error::SlotOccupied { level } => SysError::VSpaceSlotOccupied { level: level as u8 },
            vspace::Error::SlotEmpty => SysError::SlotEmpty,
        }
    }
}