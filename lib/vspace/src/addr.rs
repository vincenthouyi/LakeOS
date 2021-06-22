use crate::TableLevel;
use crate::PAGE_OFFSET;

#[derive(Copy, Clone, Debug)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Debug)]
pub struct VirtAddr(pub usize);

impl VirtAddr {
    pub fn table_index<L:TableLevel>(&self) -> usize {
        (self.0 >> (12 + 9 * (L::LEVEL - 1))) & MASK!(9)
    }
}

pub const fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    VirtAddr(paddr.0 + PAGE_OFFSET)
}

pub const fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    PhysAddr(vaddr.0 - PAGE_OFFSET)
}

impl From<VirtAddr> for PhysAddr {
    fn from(vaddr: VirtAddr) -> Self {
        virt_to_phys(vaddr)
    }
}

impl From<PhysAddr> for VirtAddr {
    fn from(paddr: PhysAddr) -> Self {
        phys_to_virt(paddr)
    }
}

impl<T> From<&T> for VirtAddr {
    fn from(t: &T) -> Self {
        Self(t as *const _ as usize)
    }
}