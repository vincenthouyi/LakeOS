#[derive(Debug, Copy, Clone, ToPrimitive, FromPrimitive, Eq, PartialEq)]
#[repr(C)]
pub enum SysError {
    OK = 0,
    CSpaceNotFound,
    CapabilityTypeError,
    LookupError,
    UnableToDerive,
    EntryNonEmpty,
    UnsupportedSyscallOp,
    VSpaceError,
    InvalidValue,

    /* Untyped */
    SizeTooSmall,

    SlotIsNotEmpty,
}

pub type SysResult<T> = core::result::Result<T, SysError>;
