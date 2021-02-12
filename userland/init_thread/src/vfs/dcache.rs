use core::slice;
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::boxed::Box;

use async_trait::async_trait;
use hashbrown::{HashMap, HashSet};
use spin::Mutex;

use naive::rpc::{self, RpcServer, ReadDirRequest, ReadDirResponse};
use naive::lmp::LmpListenerHandle;
use naive::ep_server::EP_SERVER;
use naive::path::{Path, PathBuf};
use rustyl4api::object::EpCap;

use crate::vfs::INode;

#[derive(Debug, Clone)]
pub struct DirEntry(Arc<Mutex<DirEntryImp>>);

impl DirEntry {
    pub fn new(inode: Arc<dyn INode>) -> Self {
        Self::from_inner(DirEntryImp::new(inode))
    }

    pub fn new_negative() -> Self {
        Self::from_inner(DirEntryImp::new_negative())
    }

    pub fn from_inner(inner: DirEntryImp) -> Self {
        Self(Arc::new(Mutex::new(inner)))
    }

    pub fn lookup<P: AsRef<Path>>(&self, name: P) -> Option<DirEntry> {
        self.0.lock().lookup(name)
    }

    pub fn set_inode(&self, inode: Arc<dyn INode>) {
        self.0.lock().set_inode(inode)
    }

    pub fn open(&self) -> Result<Option<usize>, ()> {
        let mut inner = self.0.lock();
        if let Some(ep) = inner.cached_ep {
            return Ok(Some(ep))
        }
        let ep = inner.open()?;
        if let Some(ep) = ep {
            inner.cached_ep = Some(ep);
            return Ok(Some(ep))
        }

        let node = DentryNode {
            dentry: self.clone()
        };

        let ep_server = EP_SERVER.try_get().unwrap();
        let (listen_badge, listen_ep) = ep_server.derive_badged_cap().unwrap();

        let listener = LmpListenerHandle::new(listen_ep.clone(), listen_badge);
        ep_server.insert_event(listen_badge, listener.clone());
        let file_svr = Box::new(RpcServer::new(listener, node));

        inner.cached_ep = Some(listen_ep.slot);
        naive::task::spawn(file_svr.run());
        Ok(Some(listen_ep.slot))
    }

    pub fn publish<P: AsRef<Path>>(&self, name: P, ep: EpCap) -> Result<(), ()> {
        self.0.lock().publish(name, ep)
    }

    pub fn remove_child<P: AsRef<Path>>(&self, name: P) {
        self.0.lock().children.remove(name.as_ref());
    }
    
    pub fn is_negative(&self) -> bool {
        self.0.lock().is_negative()
    }

    pub fn read_dir(&self) -> Result<Vec<PathBuf>, ()> {
        self.0.lock().read_dir()
    }

    pub fn cached_entries(&self) -> Vec<PathBuf> {
        self.0.lock().cached_entries()
    }

    pub fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, ()> {
        self.0
            .lock()
            .inode
            .as_mut()
            .ok_or(())?
            .read(buf, offset)
    }
}

#[derive(Debug)]
pub struct DirEntryImp {
    pub children: HashMap<PathBuf, DirEntry>,
    pub inode: Option<Arc<dyn INode>>,
    cached_ep: Option<usize>,
}

fn lookup_inode<P: AsRef<Path>>(inode: &Arc<dyn INode>, name: P) -> Option<DirEntry> {
    inode.lookup(&name)
        .map(|child_node| DirEntry::new(child_node))
}

impl DirEntryImp {
    pub fn new(inode: Arc<dyn INode>) -> Self {
        Self {
            children: HashMap::new(),
            inode: Some(inode),
            cached_ep: None
        }
    }

    pub fn new_negative() -> Self {
        Self {
            children: HashMap::new(),
            inode: None,
            cached_ep: None
        }
    }

    pub fn set_inode(&mut self, inode: Arc<dyn INode>) {
        self.inode = Some(inode);
        self.children = HashMap::new();
    }

    pub fn lookup<P: AsRef<Path>>(&mut self, name: P) -> Option<DirEntry> {
        let inode = self.inode.as_mut()?;
        Some(
            self.children
                .entry(name.as_ref().to_path_buf())
                .or_insert_with(
                    || lookup_inode(inode, name)
                        .unwrap_or(DirEntry::new_negative()))
                .clone()
        )
    }

    pub fn publish<P: AsRef<Path>>(&mut self, name: P, ep: EpCap) -> Result<(), ()> {
        self.inode
            .as_ref()
            .ok_or(())?
            .publish(&name, ep)
    }

    pub fn open(&self) -> Result<Option<usize>, ()> {
        self.inode
            .as_ref()
            .ok_or(())?
            .open()
    }

    pub fn is_negative(&self) -> bool {
        self.inode.is_none()
    }

    pub fn read_dir(&self) -> Result<Vec<PathBuf>, ()> {
        self.inode
            .as_ref()
            .ok_or(())?
            .read_dir()
    }

    pub fn cached_entries(&self) -> Vec<PathBuf> {
        self.children
            .keys()
            .cloned()
            .collect()
    }
}

struct DentryNode {
    dentry: DirEntry
}

#[async_trait]
impl rpc::RpcRequestHandlers for DentryNode {
    async fn handle_read_dir(
        &self,
        _request: &ReadDirRequest,
    ) -> rpc::Result<(ReadDirResponse, Vec<usize>)> {
        let cached_entries = self.dentry.cached_entries();
        let inode_entries = self.dentry.read_dir().map_err(|_| rpc::Error::CallNotSupported)?;
        let mut ret = HashSet::new();

        for i in cached_entries.into_iter().chain(inode_entries.into_iter()) {
            ret.insert(i);
        }
        Ok((ReadDirResponse {
            filename: ret.into_iter().collect()
        }, [].to_vec()))
    }

    async fn handle_read(&self, request: &rpc::ReadRequest) -> rpc::Result<(rpc::ReadResponse, Vec<usize>)> {
        let mut buf = Vec::with_capacity(request.len);

        unsafe {
            let buf_slice = slice::from_raw_parts_mut(buf.as_mut_ptr(), request.len);
            self.dentry
                .read(buf_slice, request.offset)
                .map(|read_len| {
                    buf.set_len(read_len);
                    (rpc::ReadResponse { buf }, [].to_vec())
                })
                .map_err(|_| rpc::Error::CallNotSupported)
        }
    }

}