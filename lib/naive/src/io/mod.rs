pub mod stdio;

pub use stdio::{stdin, stdout};
pub use futures_util::io::*;
pub use stdio::{_print};