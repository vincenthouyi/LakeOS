use crate::PAGE_OFFSET;

#[derive(Copy, Clone, Debug)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Debug)]
pub struct VirtAddr(pub usize);

impl From<VirtAddr> for PhysAddr {
    fn from(vaddr: VirtAddr) -> Self {
        PhysAddr(vaddr.0 - PAGE_OFFSET)
    }
}

impl From<PhysAddr> for VirtAddr {
    fn from(paddr: PhysAddr) -> Self {
        VirtAddr(paddr.0 + PAGE_OFFSET)
    }
}

impl<T> From<&T> for VirtAddr {
    fn from(t: &T) -> Self {
        Self(t as *const _ as usize)
    }
}