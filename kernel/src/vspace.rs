use crate::prelude::KERNEL_OFFSET;

pub use vspace::arch::*;

pub type VSpace<'a> = vspace::arch::VSpace::<'a, KERNEL_OFFSET>;
pub type VirtAddr = vspace::VirtAddr<KERNEL_OFFSET>;