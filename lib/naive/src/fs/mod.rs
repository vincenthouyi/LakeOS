mod file;
mod pwd;
mod read_dir;

pub use file::File;
pub use pwd::*;
pub use read_dir::*;

use crate::path::{Path, PathBuf};
use crate::Result;

pub fn canonicalize<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();
    if path.is_relative() {
        let mut path_buf = current_dir()?;
        path_buf.push(path);
        Ok(path_buf)
    } else {
        Ok(path.to_path_buf())
    }
}
