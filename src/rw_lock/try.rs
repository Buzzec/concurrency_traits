use crate::rw_lock::{CustomReadGuard, CustomRwLock, CustomWriteGuard};
use core::ops::{Deref, DerefMut};

/// A raw try rw lock that stores no data
pub unsafe trait RawTryRwLock {
    /// Tries to add a reader to the lock. Returns true if successful.
    fn try_add_reader(&self) -> bool;
    /// Tries to add a writer to the lock. Returns true if successful.
    fn try_add_writer(&self) -> bool;
    /// Removes a reader from this lock.
    ///
    /// # Safety
    /// Caller must ensure that this lock had a reader that was not removed
    unsafe fn remove_reader(&self);
    /// Removes a writer from this lock
    ///
    /// # Safety
    /// Caller must ensure that this lock had a writer that was not removed
    unsafe fn remove_writer(&self);
}
/// A non-blocking rwlock with try functions
///
/// ## Implementation
/// It is recommended to implement [`TryRwLockSized`] if the implement-ee can be
/// sized.
pub trait TryRwLock<'a> {
    /// The item stored by this lock
    type Item: ?Sized;
    /// The guard for reading from this lock
    type ReadGuard: Deref<Target = Self::Item>;
    /// The guard for writing to this lock
    type WriteGuard: DerefMut<Target = Self::Item>;

    /// Tries to read from the lock, returning [`None`] if cannot immediately
    fn try_read(&'a self) -> Option<Self::ReadGuard>;

    /// Tries to write to the lock, returning `None` if not able to immediately
    fn try_write(&'a self) -> Option<Self::WriteGuard>;
}
/// The functions for [`TryRwLock`] that only work for sized types.
/// Separated to allow [`TryRwLock`] to be a trait object.
pub trait TryRwLockSized<'a>: Sized + TryRwLock<'a> {
    /// Tries to read from the lock and runs `func` on the result
    ///
    /// ## Implementation
    /// Should be overwritten by implementors if can be more optimal than
    /// creating a guard
    fn try_read_func<O>(&'a self, func: impl FnOnce(Option<&Self::Item>) -> O) -> O {
        match self.try_read() {
            None => func(None),
            Some(guard) => func(Some(guard.deref())),
        }
    }
    /// Tries to write to the lock and runs `func` on the result
    ///
    /// ## Implementation
    /// Should be overwritten by implementors if can be more optimal than
    /// creating a guard
    fn try_write_func<O>(&'a self, func: impl FnOnce(Option<&mut Self::Item>) -> O) -> O {
        match self.try_write() {
            None => func(None),
            Some(mut guard) => func(Some(guard.deref_mut())),
        }
    }
}

impl<'a, T, R> TryRwLock<'a> for CustomRwLock<T, R>
where
    T: 'a,
    R: RawTryRwLock + 'a,
{
    type Item = T;
    type ReadGuard = CustomReadGuard<'a, T, R>;
    type WriteGuard = CustomWriteGuard<'a, T, R>;

    fn try_read(&'a self) -> Option<Self::ReadGuard> {
        match self.raw_lock.try_add_reader() {
            true => Some(CustomReadGuard { lock: self }),
            false => None,
        }
    }

    fn try_write(&'a self) -> Option<Self::WriteGuard> {
        match self.raw_lock.try_add_writer() {
            true => Some(CustomWriteGuard { lock: self }),
            false => None,
        }
    }
}
impl<'a, T, R> TryRwLockSized<'a> for CustomRwLock<T, R>
where
    T: 'a,
    R: RawTryRwLock + 'a,
{
    fn try_read_func<O>(&'a self, func: impl FnOnce(Option<&Self::Item>) -> O) -> O {
        match self.raw_lock.try_add_reader() {
            true => unsafe {
                let out = func(Some(&*self.data.get()));
                self.raw_lock.remove_reader();
                out
            },
            false => func(None),
        }
    }

    fn try_write_func<O>(&'a self, func: impl FnOnce(Option<&mut Self::Item>) -> O) -> O {
        match self.raw_lock.try_add_writer() {
            true => unsafe {
                let out = func(Some(&mut *self.data.get()));
                self.raw_lock.remove_writer();
                out
            },
            false => func(None),
        }
    }
}
