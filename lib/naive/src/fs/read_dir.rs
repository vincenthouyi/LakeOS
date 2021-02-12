use core::iter::Iterator;
use core::convert::AsRef;

use alloc::{
    vec::Vec,
};

use crate::path::{Path, PathBuf};

use super::File;

pub struct ReadDir {
    path: PathBuf,
    filenames: Vec<PathBuf>,
    idx: usize,
}

pub async fn read_dir<P: AsRef<Path>>(path: P) -> Result<ReadDir, ()> {
    let pathbuf = path.as_ref().to_path_buf();
    let mut fd = File::open(path).await?;
    let filenames = fd.read_dir().await?;
    Ok(ReadDir { path: pathbuf, filenames, idx: 0 })
}

impl Iterator for ReadDir {
    type Item = Result<DirEntry, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.filenames.len() {
            return None
        }

        let filename = &self.filenames[self.idx];
        self.idx += 1;

        let mut path = self.path.clone();
        path.set_file_name(filename);
        Some(Ok(DirEntry {
            path
        }))
    }
}

pub struct DirEntry {
    path: PathBuf 
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }
}