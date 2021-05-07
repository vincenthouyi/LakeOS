#![no_std]

mod raw;
mod atag;

pub use self::atag::*;

/// The address at which the firmware loads the ATAGS.
//const ATAG_BASE: usize = 0x100 + crate::KERNEL_OFFSET;

/// An iterator over the ATAGS on this system.
pub struct Atags {
    pub ptr: &'static raw::Atag,
}

impl Atags {
    /// Returns an instance of `Atags`, an iterator over ATAGS on this system.
    pub fn get(base: usize) -> Atags {
        Atags {
            ptr: unsafe { &*((0x100 + base)as *const raw::Atag) }
        }
    }
}

impl Iterator for Atags {
    type Item = Atag;

    fn next(&mut self) -> Option<Atag> {
        let ret = Some(Atag::from(self.ptr));
        self.ptr = self.ptr.next()?;
        ret
    }
}
