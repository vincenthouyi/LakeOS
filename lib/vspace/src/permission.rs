
bitflags! {
    pub struct Permission: u8 {
        const READABLE = 0b001;
        const WRITABLE = 0b010;
        const EXECUTABLE = 0b100;
        const READONLY = Self::READABLE.bits;
        const READWIRTE = Self::READABLE.bits | Self::WRITABLE.bits;
    }
}

impl Permission {
    pub fn is_readable(&self) -> bool {
        *self & Self::READABLE == Self::READABLE
    }

    pub fn is_writable(&self) -> bool {
        *self & Self::WRITABLE == Self::WRITABLE
    }

    pub fn is_executable(&self) -> bool {
        *self & Self::EXECUTABLE == Self::EXECUTABLE
    }
}