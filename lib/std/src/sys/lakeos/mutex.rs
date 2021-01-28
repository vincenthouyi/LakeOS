use rustyl4api::thread;

use crate::cell::UnsafeCell;
use crate::mem::MaybeUninit;

// pub struct Mutex {
//     inner: UnsafeCell<libc::pthread_mutex_t>,
// }

// #[inline]
// pub unsafe fn raw(m: &Mutex) -> *mut libc::pthread_mutex_t {
//     m.inner.get()
// }

// unsafe impl Send for Mutex {}
// unsafe impl Sync for Mutex {}

// #[allow(dead_code)] // sys isn't exported yet
// impl Mutex {
//     pub const fn new() -> Mutex {
//         // Might be moved to a different address, so it is better to avoid
//         // initialization of potentially opaque OS data before it landed.
//         // Be very careful using this newly constructed `Mutex`, reentrant
//         // locking is undefined behavior until `init` is called!
//         Mutex { inner: UnsafeCell::new(libc::PTHREAD_MUTEX_INITIALIZER) }
//     }
//     #[inline]
//     pub unsafe fn init(&mut self) {
//         // Issue #33770
//         //
//         // A pthread mutex initialized with PTHREAD_MUTEX_INITIALIZER will have
//         // a type of PTHREAD_MUTEX_DEFAULT, which has undefined behavior if you
//         // try to re-lock it from the same thread when you already hold a lock
//         // (https://pubs.opengroup.org/onlinepubs/9699919799/functions/pthread_mutex_init.html).
//         // This is the case even if PTHREAD_MUTEX_DEFAULT == PTHREAD_MUTEX_NORMAL
//         // (https://github.com/rust-lang/rust/issues/33770#issuecomment-220847521) -- in that
//         // case, `pthread_mutexattr_settype(PTHREAD_MUTEX_DEFAULT)` will of course be the same
//         // as setting it to `PTHREAD_MUTEX_NORMAL`, but not setting any mode will result in
//         // a Mutex where re-locking is UB.
//         //
//         // In practice, glibc takes advantage of this undefined behavior to
//         // implement hardware lock elision, which uses hardware transactional
//         // memory to avoid acquiring the lock. While a transaction is in
//         // progress, the lock appears to be unlocked. This isn't a problem for
//         // other threads since the transactional memory will abort if a conflict
//         // is detected, however no abort is generated when re-locking from the
//         // same thread.
//         //
//         // Since locking the same mutex twice will result in two aliasing &mut
//         // references, we instead create the mutex with type
//         // PTHREAD_MUTEX_NORMAL which is guaranteed to deadlock if we try to
//         // re-lock it from the same thread, thus avoiding undefined behavior.
//         let mut attr = MaybeUninit::<libc::pthread_mutexattr_t>::uninit();
//         let r = libc::pthread_mutexattr_init(attr.as_mut_ptr());
//         debug_assert_eq!(r, 0);
//         let r = libc::pthread_mutexattr_settype(attr.as_mut_ptr(), libc::PTHREAD_MUTEX_NORMAL);
//         debug_assert_eq!(r, 0);
//         let r = libc::pthread_mutex_init(self.inner.get(), attr.as_ptr());
//         debug_assert_eq!(r, 0);
//         let r = libc::pthread_mutexattr_destroy(attr.as_mut_ptr());
//         debug_assert_eq!(r, 0);
//     }
//     #[inline]
//     pub unsafe fn lock(&self) {
//         let r = libc::pthread_mutex_lock(self.inner.get());
//         debug_assert_eq!(r, 0);
//     }
//     #[inline]
//     pub unsafe fn unlock(&self) {
//         let r = libc::pthread_mutex_unlock(self.inner.get());
//         debug_assert_eq!(r, 0);
//     }
//     #[inline]
//     pub unsafe fn try_lock(&self) -> bool {
//         libc::pthread_mutex_trylock(self.inner.get()) == 0
//     }
//     #[inline]
//     #[cfg(not(target_os = "dragonfly"))]
//     pub unsafe fn destroy(&self) {
//         let r = libc::pthread_mutex_destroy(self.inner.get());
//         debug_assert_eq!(r, 0);
//     }
//     #[inline]
//     #[cfg(target_os = "dragonfly")]
//     pub unsafe fn destroy(&self) {
//         let r = libc::pthread_mutex_destroy(self.inner.get());
//         // On DragonFly pthread_mutex_destroy() returns EINVAL if called on a
//         // mutex that was just initialized with libc::PTHREAD_MUTEX_INITIALIZER.
//         // Once it is used (locked/unlocked) or pthread_mutex_init() is called,
//         // this behaviour no longer occurs.
//         debug_assert!(r == 0 || r == libc::EINVAL);
//     }
// }

use core::sync::atomic::{AtomicUsize, Ordering};
pub struct ReentrantMutex {
    inner: AtomicUsize,
}

impl ReentrantMutex {
    pub const unsafe fn uninitialized() -> ReentrantMutex {
        Self{ inner: AtomicUsize::new(0) }
    }

    pub unsafe fn init(&self) {
    }

    pub unsafe fn lock(&self) {
        while let false = self.try_lock() { } //TODO: yield thread
    }

    #[inline]
    pub unsafe fn try_lock(&self) -> bool {
        let thread_id = thread::thread_id();

        let out = self.inner.compare_and_swap(0, thread_id, Ordering::Relaxed);
        
        if out == 0 {
            return true;
        }

        if self.inner.load(Ordering::Relaxed) == thread_id {
            return true;
        }

        false
    }

    pub unsafe fn unlock(&self) {
        let thread_id = thread::thread_id();

        self.inner.compare_and_swap(thread_id, 0, Ordering::Relaxed);
    }

    pub unsafe fn destroy(&self) {
        // let result = libc::pthread_mutex_destroy(self.inner.get());
        // debug_assert_eq!(result, 0);
    }
}
