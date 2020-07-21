use crate::io::{self};
use crate::mem::ManuallyDrop;
// use crate::sys::fd::FileDesc;

// pub struct Stdin(());
pub struct Stdout(());
pub struct Stderr(());

// impl Stdin {
//     pub fn new() -> io::Result<Stdin> {
//         Ok(Stdin(()))
//     }
// }

// impl io::Read for Stdin {
//     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
//         ManuallyDrop::new(FileDesc::new(libc::STDIN_FILENO)).read(buf)
//     }

//     fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
//         ManuallyDrop::new(FileDesc::new(libc::STDIN_FILENO)).read_vectored(bufs)
//     }

//     #[inline]
//     fn is_read_vectored(&self) -> bool {
//         true
//     }
// }

impl Stdout {
    pub fn new() -> io::Result<Stdout> {
        Ok(Stdout(()))
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        use core::fmt::Write;
        use core::str::from_utf8;
        use rustyl4api::debug_printer::debug_printer; 

        let string = from_utf8(buf).unwrap();
        for c in string.chars() {
            use rustyl4api::object::EpCap;
            use rustyl4api::process::ProcessCSpace;
            let ep = EpCap::new(ProcessCSpace::Stdout as usize);
            ep.send(&[c as usize]);

        }
        Ok(buf.len())
    }

    // // fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
    // //     ManuallyDrop::new(FileDesc::new(libc::STDOUT_FILENO)).write_vectored(bufs)
    // // }

    // #[inline]
    // fn is_write_vectored(&self) -> bool {
    //     true
    // }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Stderr {
    pub fn new() -> io::Result<Stderr> {
        Ok(Stderr(()))
    }
}

impl io::Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        use core::fmt::Write;
        use core::str::from_utf8;
        use rustyl4api::debug_printer::debug_printer; 

        let string = from_utf8(buf).unwrap();
        for c in string.chars() {
            use rustyl4api::object::EpCap;
            use rustyl4api::process::ProcessCSpace;
            let ep = EpCap::new(ProcessCSpace::Stderr as usize);
            ep.send(&[c as usize]);
        }
        Ok(buf.len())
    }

    // fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
    //     ManuallyDrop::new(FileDesc::new(libc::STDERR_FILENO)).write_vectored(bufs)
    // }

    // #[inline]
    // fn is_write_vectored(&self) -> bool {
    //     true
    // }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn is_ebadf(err: &io::Error) -> bool {
    // err.raw_os_error() == Some(libc::EBADF as i32)
    false
}

// pub const STDIN_BUF_SIZE: usize = crate::sys_common::io::DEFAULT_BUF_SIZE;

// pub fn panic_output() -> Option<impl io::Write> {
//     Stderr::new().ok()
// }
