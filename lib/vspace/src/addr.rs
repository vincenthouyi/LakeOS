use crate::TableLevel;

#[derive(Copy, Clone, Debug)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Debug)]
pub struct VirtAddr<const O: usize>(pub usize);

impl<const O: usize> VirtAddr<O> {
    pub fn new(inner: usize) -> Self {
        Self(inner)
    }

    pub fn table_index<L: TableLevel>(&self) -> usize {
        (self.0 >> (12 + 9 * (L::LEVEL - 1))) & MASK!(9)
    }
}

pub const fn phys_to_virt<const O: usize>(paddr: PhysAddr) -> VirtAddr<O> {
    VirtAddr(paddr.0 + O)
}

pub const fn virt_to_phys<const O: usize>(vaddr: VirtAddr<O>) -> PhysAddr {
    PhysAddr(vaddr.0 - O)
}

impl<const O: usize> From<VirtAddr<O>> for PhysAddr {
    fn from(vaddr: VirtAddr<O>) -> Self {
        virt_to_phys(vaddr)
    }
}

impl<const O: usize> From<PhysAddr> for VirtAddr<O> {
    fn from(paddr: PhysAddr) -> Self {
        phys_to_virt(paddr)
    }
}

impl<T, const O: usize> From<&T> for VirtAddr<O> {
    fn from(t: &T) -> Self {
        Self(t as *const _ as usize)
    }
}
