use core::convert::AsRef;

use alloc::boxed::Box;
use alloc::sync::Arc;

use hashbrown::{HashMap};

use rustyl4api::object::{EpCap, CNodeCap};
use naive::path::{Path, PathBuf, Component};

pub use dcache::*;
pub use inode::*;

mod dcache;
mod inode;

#[derive(Debug)]
pub struct Vfs {
    mount_table: HashMap<PathBuf, Box<dyn FileSystem>>,
    root: DirEntry,
}

impl Vfs {
    pub fn new() -> Self {
        Self {
            mount_table: HashMap::new(),
            root: DirEntry::new_negative(),
        }
    }

    fn lookup<P: AsRef<Path>>(&mut self, path: P) -> Option<DirEntry> {
        let mut cur_dentry = self.root.clone();
        let mut components = path.as_ref().components();
        while let Some(compo) = components.next() {
            if let Component::Normal(name) = compo {
                cur_dentry = cur_dentry.lookup(name)?;
            }
        }
        Some(cur_dentry)
    }

    pub fn mount<T: 'static + FileSystem, P: AsRef<Path>>(&mut self, path: P, fs: T) -> Result<(), ()> {
        let entry = self.lookup(path.as_ref()).ok_or(())?;
        entry.set_inode(fs.root());
        self.mount_table.insert(path.as_ref().to_path_buf(), Box::new(fs));
        Ok(())
    }

    // pub fn unmount() {

    // }

    pub fn open<P: AsRef<Path>>(&mut self, path: P) -> Result<IndexNode, ()> {
        let entry = self.lookup(path).ok_or(())?;
        if entry.is_negative() {
            return Err(())
        }
        let ep = entry.open()?.ok_or(())?;
        let copy_slot = naive::space_manager::gsm!().cspace_alloc().unwrap();
        let cspace = CNodeCap::new(rustyl4api::init::InitCSpaceSlot::InitCSpace as usize);
        cspace
            .cap_copy(
                copy_slot,
                ep,
            )
            .unwrap();
        Ok(IndexNode {
            cap: copy_slot,
            node_type: NodeType::File,
        })
    }

    pub fn publish<P: AsRef<Path>>(&mut self, path: P, ep: EpCap) -> Result<(), ()> {
        let parent = path.as_ref().parent().ok_or(())?;
        let filename= path.as_ref().file_stem().ok_or(())?;
        let parent_dentry = self.lookup(parent).ok_or(())?;
        let ret = parent_dentry.publish(filename, ep)?;
        parent_dentry.remove_child(filename);
        Ok(ret)
    }
}

pub enum NodeType {
    File,
    Directory
}

pub struct IndexNode {
    pub cap: usize,
    pub node_type: NodeType,
}

pub trait FileSystem: Send + Sync + core::fmt::Debug {
    fn root(&self) -> Arc<dyn INode>;

    fn publish(&self, _path: &Path, _ep: EpCap) -> Result<(), ()> {
        Err(())
    }
}
