
mod error;

pub use error::ErrorKind;

pub type Result<T> = core::result::Result<T, ErrorKind>;