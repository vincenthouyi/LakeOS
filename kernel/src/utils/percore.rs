
pub struct PerCore<T, const N: usize>(pub [T; N]);

unsafe impl<T, const N: usize> core::marker::Sync for PerCore<T, N> { }

impl<T, const N: usize> PerCore<T, N> {
    pub fn get(&self) -> &T {
        let i = crate::arch::affinity();

        &self.0[i]
    }

    pub fn get_mut(&mut self) -> &mut T {
        let i = crate::arch::affinity();
        &mut self.0[i]
    }

    pub fn get_mut_unsafe(&self) -> &mut T {
        let i = crate::arch::affinity();
        unsafe{ &mut *(&self.0[i] as *const T as *mut T) }
    }
}

impl<T, const N: usize> core::ops::Deref for PerCore<T, N> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T, const N: usize> core::ops::DerefMut for PerCore<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}