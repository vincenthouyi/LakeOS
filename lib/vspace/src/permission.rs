
bitflags! {
    pub struct Permission: u8 {
        const READABLE = 0b001;
        const WRITABLE = 0b010;
        const EXECUTABLE = 0b100;
    }
}

impl Permission {
    pub fn new(readable: bool, writable: bool, executable: bool) -> Self {
        let mut x = Permission { bits: 0 };
        if readable {
            x |= Permission::READABLE;
        }
        if writable {
            x |= Permission::WRITABLE;
        }
        if executable {
            x |= Permission::EXECUTABLE;
        }
        x
    }

    pub fn writable() -> Self {
        Self::READABLE | Self::WRITABLE
    }

    pub fn readonly() -> Self {
        Self::READABLE
    }

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

impl Into<usize> for Permission {
    fn into(self: Permission) -> usize {
        self.bits as usize
    }
}

impl From<usize> for Permission {
    fn from(x: usize) -> Permission {
        Permission {
            bits: x as u8
        }
    }
}