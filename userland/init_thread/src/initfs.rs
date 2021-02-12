use alloc::{
    vec::Vec,
    sync::Arc,
};

use naive::{
    path::{Path, PathBuf},
    os_str::{OsStr, OsStrExt}
};

use crate::vfs;

#[derive(Debug, Clone)]
pub struct InitFs {
    archive: Arc<cpio::NewcReader<'static>>
}

impl InitFs {
    pub fn new() -> Self {
        let archive = unsafe {
            cpio::NewcReader::from_bytes(
                core::slice::from_raw_parts(0x40000000 as *const u8, 0x4000000)
            )
        };
        Self { archive: Arc::new(archive) }
    }

    pub fn files(&self) -> Vec<PathBuf> {
        self.archive
            .entries()
            .map(|e| Path::new(OsStr::from_bytes(&e.name())).to_path_buf())
            .collect()
    }

    pub fn get(&self, name: &[u8]) -> Option<&'static [u8]> {
        self.archive
            .entries()
            .find(|e| e.name() == name)
            .map(|e| e.content())
    }
}

impl vfs::FileSystem for InitFs {
    fn root(&self) -> Arc<dyn vfs::INode> {
        Arc::new(Dir {
            path: Path::new("").to_path_buf(),
            fs: self.clone()
        })
    }
}

#[derive(Debug)]
pub struct File {
    data: &'static [u8],
}

impl File {
    pub fn new(data: &'static [u8]) -> Self {
        Self { data }
    }
}

impl vfs::INode for File {
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, ()> {
        let len = self.data[offset..].len().min(buf.len());
        buf[..len].copy_from_slice(&self.data[offset..offset+len]);
        Ok(len)
    }
}

#[derive(Debug)]
pub struct Dir {
    path: PathBuf,
    fs: InitFs,
}

impl vfs::INode for Dir {
    fn lookup(&self, name: &dyn AsRef<Path>) -> Option<Arc<dyn vfs::INode>> {
        let file = self.fs
            .archive
            .entries()
            .find(|ent| OsStr::from_bytes(&ent.name()) == self.path.with_file_name(name.as_ref()))?;
        Some(Arc::new(File::new(file.content())))
    }

    fn read_dir(&self) -> Result<Vec<PathBuf>, ()> {
        let entries = self.fs
            .files()
            .into_iter()
            .filter(|name| name.starts_with(&self.path))
            .collect();
        Ok(entries)
    }
}