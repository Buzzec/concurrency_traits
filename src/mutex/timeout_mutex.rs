use core::time::Duration;
use crate::mutex::{TryMutexSized, TryMutex, RawMutex, CustomMutex, CustomMutexGuard};
use core::ops::DerefMut;

/// A raw mutex that can be timed out and holds no data.
pub unsafe trait RawTimeoutMutex: RawMutex {
    /// Locks the mutex on a timeout. Returns true if locked.
    fn lock_timeout(&self, timeout: Duration) -> bool;
}
/// A mutex that can timeout for locking
///
/// ## Implementation
/// It is recommended to implement [`TimeoutMutexSized`] if the implement-ee can
/// be sized.
pub trait TimeoutMutex<'a>: TryMutex<'a> {
    /// Locks the mutex blocking for timeout or until locked
    fn lock_timeout(&'a self, timeout: Duration) -> Option<Self::Guard>;
}
/// The functions for [`TimeoutMutex`] that only work for sized types.
/// Separated to allow [`TimeoutMutex`] to be a trait object.
pub trait TimeoutMutexSized<'a>: Sized + TimeoutMutex<'a> + TryMutexSized<'a> {
    /// Attempts to lock the mutex before timeout has passed and runs func on
    /// the result
    ///
    /// ## Implementation
    /// Should be overwritten by implementors if can be more optimal than
    /// creating a guard
    fn lock_timeout_func<O>(
        &'a self,
        timeout: Duration,
        func: impl FnOnce(Option<&mut Self::Item>) -> O,
    ) -> O {
        match self.lock_timeout(timeout) {
            None => func(None),
            Some(mut guard) => func(Some(guard.deref_mut())),
        }
    }
}

impl<'a, T, M> TimeoutMutex<'a> for CustomMutex<T, M>
where
    T: 'a,
    M: RawTimeoutMutex + 'a,
{
    fn lock_timeout(&'a self, timeout: Duration) -> Option<Self::Guard> {
        match self.raw_mutex.lock_timeout(timeout) {
            true => Some(CustomMutexGuard { mutex: self }),
            false => None,
        }
    }
}
impl<'a, T, M> TimeoutMutexSized<'a> for CustomMutex<T, M>
where
    T: 'a,
    M: RawTimeoutMutex + 'a,
{
    fn lock_timeout_func<O>(
        &'a self,
        timeout: Duration,
        func: impl FnOnce(Option<&mut Self::Item>) -> O,
    ) -> O {
        match self.raw_mutex.lock_timeout(timeout) {
            true => unsafe {
                let out = func(Some(&mut *self.data.get()));
                self.raw_mutex.unlock();
                out
            },
            false => func(None),
        }
    }
}
