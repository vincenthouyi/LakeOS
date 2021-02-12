use conquer_once::spin::OnceCell;
use spin::Mutex;

use crate::path::{Path, PathBuf};

static PWD: OnceCell<Mutex<PathBuf>> = OnceCell::uninit();


pub fn current_dir() -> Result<PathBuf, ()> {
    let pwd= PWD.try_get_or_init(|| Mutex::new(Path::new("/").into()));
    Ok(pwd.unwrap().lock().clone())
}

pub async fn set_current_dir<P: AsRef<Path>>(path: P) -> Result<(), ()> {
    let pwd = PWD.try_get_or_init(|| Mutex::new(path.as_ref().into())).unwrap();
    *pwd.lock() = path.as_ref().into();
    Ok(())
}