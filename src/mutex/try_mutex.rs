use core::ops::DerefMut;
use crate::mutex::{CustomMutex, CustomMutexGuard};

/// A raw mutex that can be tried and holds no data.
pub unsafe trait RawTryMutex {
    /// Locks the mutex, non-blocking. Returns true if locked.
    fn try_lock(&self) -> bool;
    /// # Safety
    /// Must only be called when sure that no references exist to contained
    /// data.
    unsafe fn unlock(&self);
}
/// A non-blocking mutex with try functions.
///
/// ## Implementation
/// It is recommended to implement [`TryMutexSized`] if the implement-ee can be
/// sized.
pub trait TryMutex<'a> {
    /// The item stored in the mutex
    type Item: ?Sized;
    /// The guard for the mutex
    type Guard: DerefMut<Target = Self::Item>;

    /// Tries to lock the mutex, returning `None` if not possible.
    fn try_lock(&'a self) -> Option<Self::Guard>;
}
/// The functions for [`TryMutex`] that only work for sized types.
/// Separated to allow [`TryMutex`] to be a trait object.
pub trait TryMutexSized<'a>: Sized + TryMutex<'a> {
    /// Runs the function the value in the mutex if available immediately.
    ///
    /// ## Implementation
    /// Should be overwritten by implementors if can be more optimal than
    /// creating a guard
    fn try_lock_func<O>(&'a self, func: impl FnOnce(Option<&mut Self::Item>) -> O) -> O {
        match self.try_lock() {
            None => func(None),
            Some(mut guard) => func(Some(guard.deref_mut())),
        }
    }
}

impl<'a, T, M> TryMutex<'a> for CustomMutex<T, M>
where
    T: 'a,
    M: RawTryMutex + 'a,
{
    type Item = T;
    type Guard = CustomMutexGuard<'a, T, M>;

    fn try_lock(&'a self) -> Option<Self::Guard> {
        match self.raw_mutex.try_lock() {
            true => Some(CustomMutexGuard { mutex: self }),
            false => None,
        }
    }
}
impl<'a, T, M> TryMutexSized<'a> for CustomMutex<T, M>
where
    T: 'a,
    M: RawTryMutex + 'a,
{
    fn try_lock_func<O>(&'a self, func: impl FnOnce(Option<&mut Self::Item>) -> O) -> O {
        match self.raw_mutex.try_lock() {
            true => unsafe {
                let out = func(Some(&mut *self.data.get()));
                self.raw_mutex.unlock();
                out
            },
            false => func(None),
        }
    }
}
