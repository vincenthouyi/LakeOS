
pub struct PerCore<T, const N: usize>(pub [T; N]);

unsafe impl<T, const N: usize> core::marker::Sync for PerCore<T, N> { }

impl<T, const N: usize> PerCore<T, N> {
    pub unsafe fn get_unsafe(&self, i: usize) -> &T {
        &self.0[i]
    }
}

impl<T, const N: usize> core::ops::Deref for PerCore<T, N> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let i = crate::arch::affinity();

        &self.0[i]
    }
}

impl<T, const N: usize> core::ops::DerefMut for PerCore<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let i = crate::arch::affinity();
        &mut self.0[i]
    }
}