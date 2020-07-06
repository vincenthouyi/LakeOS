
pub const FRAME_BIT_SIZE: usize = 12;
pub const FRAME_SIZE: usize = 1 << FRAME_BIT_SIZE;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Permission {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}

impl Permission {
    pub const fn new(readable: bool, writable: bool, executable: bool) -> Self {
        Self {
            readable: readable,
            writable: writable,
            executable: executable,
        }
    }

    pub const fn writable() -> Self {
        Self::new(true, true, false)
    }

    pub const fn readonly() -> Self {
        Self::new(true, false, false)
    }

    pub const fn executable() -> Self {
        Self::new(true, false, true)
    }
}

impl core::convert::Into<usize> for Permission {
    fn into(self) -> usize {
        let r = self.readable as usize;
        let w = self.writable as usize;
        let x = self.executable as usize;

        r << 2 | w << 1 | x << 0
    }
}

impl core::convert::From<usize> for Permission {
    fn from(p: usize) -> Self {
        Self::new(p >> 2 == 1,
                  (p >> 1) & 1 == 1,
                  p & 1 == 1)
    }
}