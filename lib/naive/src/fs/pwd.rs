use spin::Mutex;

use crate::path::{Path, PathBuf};

lazy_static! {
    static ref PWD: Mutex<PathBuf> = {
        Mutex::new(Path::new("/").into())
    };
}

pub fn current_dir() -> Result<PathBuf, ()> {
    Ok(PWD.lock().clone())
}

pub async fn set_current_dir<P: AsRef<Path>>(path: P) -> Result<(), ()> {
    *PWD.lock() = path.as_ref().into();
    Ok(())
}
