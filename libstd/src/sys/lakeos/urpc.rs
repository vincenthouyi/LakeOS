use crate::io::{self, ErrorKind};
use naive::urpc as imp;

pub use imp::stream::{UrpcStream, UrpcStreamHandle};

#[unstable(feature = "alloc_internals", issue = "none")]
impl io::Write for UrpcStreamHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_bytes(buf)
            .map_err(|e| {
                let errkind: io::ErrorKind = e.into();
                errkind.into()
            })
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[unstable(feature = "alloc_internals", issue = "none")]
impl io::Read for UrpcStreamHandle {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.read_bytes(buf)
            .map_err(|e| {
                let errkind: ErrorKind = e.into();
                errkind.into()
            })
    }
}