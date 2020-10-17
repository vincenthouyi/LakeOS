use core::cell::UnsafeCell;

pub struct PerCore<T, const N: usize>(pub [UnsafeCell<T>; N]);

unsafe impl<T, const N: usize> core::marker::Sync for PerCore<T, N> { }

impl<T, const N: usize> PerCore<T, N> {
    pub unsafe fn get_unsafe(&self, i: usize) -> &T {
        &*self.0[i].get()
    }

    pub fn get(&self) -> &T {
        let i = crate::arch::affinity();
        unsafe{
            &*self.0[i].get()
        }
    }

    pub fn get_mut(&self) -> &mut T {
        let i = crate::arch::affinity();
        unsafe {
            &mut *self.0[i].get()
        }
    }
}