use naive::objects::EpRef;
use naive::path::{Path, PathBuf};

use alloc::sync::Arc;
use alloc::vec::Vec;

pub trait INode: Send + Sync + core::fmt::Debug {
    fn open(&self) -> Result<Option<EpRef>, ()> {
        Ok(None)
    }

    fn lookup(&self, _name: &dyn AsRef<Path>) -> Option<Arc<dyn INode>> {
        None
    }

    fn publish(&self, _name: &dyn AsRef<Path>, _ep: EpRef) -> Result<(), ()> {
        Err(())
    }

    fn read_dir(&self) -> Result<Vec<PathBuf>, ()> {
        Err(())
    }

    fn read(&self, _buf: &mut [u8], _offset: usize) -> Result<usize, ()> {
        Err(())
    }
}
