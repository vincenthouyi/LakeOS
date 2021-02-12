use alloc::sync::Arc;
use alloc::vec::Vec;

use hashbrown::HashMap;
use spin::Mutex;

use rustyl4api::object::EpCap;
use naive::path::{Path, PathBuf};

use crate::vfs::{self, INode};


#[derive(Debug, Clone)]
pub struct DevFs {
    nodes: Arc<Mutex<HashMap<PathBuf, DevNode>>>,
}

impl DevFs {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(Mutex::new(HashMap::new()))
        }
    }
}

impl vfs::FileSystem for DevFs {
    fn root(&self) -> Arc<dyn vfs::INode> {
        Arc::new(Dir::new(self.clone()))
    }

    fn publish(&self, path: &Path, ep: EpCap) -> Result<(), ()> {
        self.nodes.lock().insert(path.to_path_buf(), DevNode{ ep });
        Ok(())
    }
}

#[derive(Debug)]
pub struct DevNode {
    ep: EpCap,
}

impl INode for DevNode {
    fn open(&self) -> Result<Option<usize>, ()> {
        Ok(Some(self.ep.slot))
    }
}

#[derive(Debug)]
struct Dir {
    fs: DevFs,
}

impl Dir {
    pub fn new(fs: DevFs) -> Self {
        Self { fs }
    }
}

impl INode for Dir {
    fn lookup(&self, name: &dyn AsRef<Path>) -> Option<Arc<dyn INode>> {
        let dev_guard = self.fs.nodes.lock();
        let node = dev_guard.get(&name.as_ref().to_path_buf())?;
        Some(Arc::new(DevNode {
            ep: node.ep.clone()
        }))
    }

    fn publish(&self, name: &dyn AsRef<Path>, ep: EpCap) -> Result<(), ()> {
        self.fs
            .nodes
            .lock()
            .insert(name.as_ref().to_path_buf(), DevNode { ep });
        Ok(())
    }

    fn read_dir(&self) -> Result<Vec<PathBuf>, ()> {
        let entries = self.fs
            .nodes
            .lock()
            .keys()
            .cloned()
            .collect();
        Ok(entries)
    }
}