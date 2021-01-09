use crate::objects::{TcbObj, TCB_OBJ_BIT_SZ};
use core::cell::Cell;
use core::ptr::NonNull;

#[derive(Debug, Default)]
pub struct TcbQueueNode {
    prev: Cell<Option<NonNull<TcbQueueNode>>>,
    next: Cell<Option<NonNull<TcbQueueNode>>>,
}

impl TcbQueueNode {
    pub const fn new() -> Self {
        Self {
            prev: Cell::new(None),
            next: Cell::new(None),
        }
    }

    pub unsafe fn tcb(&self) -> &TcbObj {
        let addr = self as *const _ as usize;
        let tcb_addr = addr & !MASK!(TCB_OBJ_BIT_SZ);

        &*(tcb_addr as *const TcbObj)
    }

    pub unsafe fn tcb_mut(&self) -> &mut TcbObj {
        let addr = self as *const _ as usize;
        let tcb_addr = addr & !MASK!(TCB_OBJ_BIT_SZ);

        &mut *(tcb_addr as *mut TcbObj)
    }

    pub fn get_prev<'a>(&self) -> Option<&'a TcbQueueNode> {
        self.prev.get().map(|p| unsafe { &*p.as_ptr() })
    }

    pub fn get_next<'a>(&self) -> Option<&'a TcbQueueNode> {
        self.next.get().map(|p| unsafe { &*p.as_ptr() })
    }

    pub fn set_prev(&self, prev: Option<&TcbQueueNode>) {
        self.prev.set(prev.map(|x| x.into()));
    }

    pub fn set_next(&self, next: Option<&TcbQueueNode>) {
        self.next.set(next.map(|x| x.into()));
    }

    //    pub fn append(&mut self, next: &mut TcbQueueNode) {
    //        next.next = self.next.replace(next.into());
    //        next.next.map(|mut p| unsafe{
    //            p.as_mut()
    //             .prev
    //             .replace(next.into())
    //        });
    //        next.prev.replace(self.into());
    //        if self.prev.is_none() {
    //            self.prev = Some(next.into());
    //        }
    //    }

    pub fn prepend(&self, node: &TcbQueueNode) {
        let old_prev = self.get_prev().unwrap_or(self);
        old_prev.set_next(Some(node.into()));
        node.set_next(Some(self));
        node.set_prev(Some(old_prev));
        self.set_prev(Some(node.into()));
    }

    //    pub fn pop_prev<'a>(&self) -> Option<&'a mut TcbQueueNode> {
    //        unimplemented!()
    //    }

    pub unsafe fn pop_next<'a>(&self) -> Option<&'a TcbQueueNode> {
        self.get_next().map(|node| {
            node.detach();
            node
        })
    }

    pub fn detach(&self) {
        if self.prev.get().is_none() {
            return;
        }

        if self.prev == self.next {
            self.get_prev().unwrap().set_prev(None);
            self.get_next().unwrap().set_next(None);
        } else {
            self.get_prev().unwrap().set_next(self.get_next());
            self.get_next().unwrap().set_prev(self.get_prev());
        }

        self.set_prev(None);
        self.set_next(None);
    }
}

#[derive(Debug, Default)]
pub struct TcbQueue {
    node: TcbQueueNode,
}

impl TcbQueue {
    pub const fn new() -> Self {
        Self {
            node: TcbQueueNode::new(),
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.node.next.get().is_none()
    }

    pub fn enqueue(&self, tcb: &TcbObj) {
        self.node.prepend(&tcb.node)
    }

    pub fn dequeue<'a>(&self) -> Option<&'a mut TcbObj> {
        unsafe { self.node.pop_next().map(|x| x.tcb_mut()) }
    }

    pub fn head<'a>(&self) -> Option<&'a TcbObj> {
        unsafe { self.node.get_next().map(|x| x.tcb()) }
    }

    pub fn head_mut<'a>(&self) -> Option<&'a mut TcbObj> {
        unsafe { self.node.get_next().map(|x| x.tcb_mut()) }
    }

    #[allow(dead_code)]
    pub fn tail<'a>(&self) -> Option<&'a TcbObj> {
        unsafe { self.node.get_prev().map(|x| x.tcb()) }
    }
}
