use alloc::sync::Arc;
use alloc::vec::Vec;

use naive::path::PathBuf;

use crate::vfs::{self, INode};

#[derive(Debug, Clone)]
pub struct RootFs {}

impl RootFs {
    pub fn new() -> Self {
        Self {}
    }
}

impl vfs::FileSystem for RootFs {
    fn root(&self) -> Arc<dyn vfs::INode> {
        Arc::new(DirNode::new(self.clone()))
    }
}

#[derive(Debug)]
struct DirNode {
    fs: RootFs,
}

impl DirNode {
    pub fn new(fs: RootFs) -> Self {
        Self { fs }
    }
}

impl INode for DirNode {
    fn read_dir(&self) -> Result<Vec<PathBuf>, ()> {
        Ok(Vec::new())
    }
}
