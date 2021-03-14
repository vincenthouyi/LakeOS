pub mod stdio;

pub use futures_util::io::*;
pub use stdio::_print;
pub use stdio::{set_stdin, set_stdout, stdin, stdout};
