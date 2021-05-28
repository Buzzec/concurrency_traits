use crate::rw_lock::{
    CustomReadGuard, CustomRwLock, CustomWriteGuard, RawTryRwLock, TryRwLock, TryRwLockSized,
};
use core::ops::{Deref, DerefMut};
use core::time::Duration;

/// A raw timeout rw lock that stores no data
pub unsafe trait RawTimeoutRwLock: RawTryRwLock {
    /// Adds a reader to this lock with a timeout. Returns true if successful
    fn add_reader_timeout(&self, timeout: Duration) -> bool;
    /// Adds a writer to this lock with a timeout. Returns true if successful
    fn add_writer_timeout(&self, timeout: Duration) -> bool;
}
/// An RwLock that can be timed out on
///
/// ## Implementation
/// It is recommended to implement [`TimeoutRwLockSized`] if the implement-ee
/// can be sized.
pub trait TimeoutRwLock<'a>: TryRwLock<'a> {
    /// Reads from the lock with a timeout
    fn read_timeout(&'a self, timeout: Duration) -> Option<Self::ReadGuard>;
    /// Writes to the lock with a timeout
    fn write_timeout(&'a self, timeout: Duration) -> Option<Self::WriteGuard>;
}
/// The functions for [`TimeoutRwLock`] that only work for sized types.
/// Separated to allow [`TimeoutRwLock`] to be a trait object.
pub trait TimeoutRwLockSized<'a>: Sized + TimeoutRwLock<'a> + TryRwLockSized<'a> {
    /// Reads from the lock with a timeout running `func` on the result
    ///
    /// ## Implementation
    /// Should be overwritten by implementors if can be more optimal than
    /// creating a guard
    fn read_timeout_func<O>(
        &'a self,
        timeout: Duration,
        func: impl FnOnce(Option<&Self::Item>) -> O,
    ) -> O {
        match self.read_timeout(timeout) {
            None => func(None),
            Some(guard) => func(Some(guard.deref())),
        }
    }

    /// Writes to the lock with a timeout running `func` on the result
    ///
    /// ## Implementation
    /// Should be overwritten by implementors if can be more optimal than
    /// creating a guard
    fn write_timeout_func<O>(
        &'a self,
        timeout: Duration,
        func: impl FnOnce(Option<&mut Self::Item>) -> O,
    ) -> O {
        match self.write_timeout(timeout) {
            None => func(None),
            Some(mut guard) => func(Some(guard.deref_mut())),
        }
    }
}

impl<'a, T, R> TimeoutRwLock<'a> for CustomRwLock<T, R>
where
    T: 'a,
    R: RawTimeoutRwLock + 'a,
{
    fn read_timeout(&'a self, timeout: Duration) -> Option<Self::ReadGuard> {
        match self.raw_lock.add_reader_timeout(timeout) {
            true => Some(CustomReadGuard { lock: self }),
            false => None,
        }
    }

    fn write_timeout(&'a self, timeout: Duration) -> Option<Self::WriteGuard> {
        match self.raw_lock.add_writer_timeout(timeout) {
            true => Some(CustomWriteGuard { lock: self }),
            false => None,
        }
    }
}
impl<'a, T, R> TimeoutRwLockSized<'a> for CustomRwLock<T, R>
where
    T: 'a,
    R: RawTimeoutRwLock + 'a,
{
    fn read_timeout_func<O>(
        &'a self,
        timeout: Duration,
        func: impl FnOnce(Option<&Self::Item>) -> O,
    ) -> O {
        match self.raw_lock.add_reader_timeout(timeout) {
            true => unsafe {
                let out = func(Some(&*self.data.get()));
                self.raw_lock.remove_reader();
                out
            },
            false => func(None),
        }
    }

    fn write_timeout_func<O>(
        &'a self,
        timeout: Duration,
        func: impl FnOnce(Option<&mut Self::Item>) -> O,
    ) -> O {
        match self.raw_lock.add_writer_timeout(timeout) {
            true => unsafe {
                let out = func(Some(&mut *self.data.get()));
                self.raw_lock.remove_writer();
                out
            },
            false => func(None),
        }
    }
}
