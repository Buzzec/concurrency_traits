//! Traits for RwLocks

mod atomic_rw_lock;
pub use atomic_rw_lock::*;

#[cfg(feature = "alloc")]
mod rw_lock_alloc;
#[cfg(feature = "std")]
mod rw_lock_std;

#[cfg(feature = "alloc")]
pub use rw_lock_alloc::*;

use core::cell::UnsafeCell;
use core::future::Future;
use core::ops::{Deref, DerefMut};
use core::time::Duration;

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

/// A generic blocking reader-writer lock trait
///
/// ## Implementation
/// It is recommended to implement [`RwLockSized`] if the implement-ee can be
/// sized.
pub trait RwLock<'a>: TryRwLock<'a> {
    /// Reads from the lock, blocking until able.
    fn read(&'a self) -> Self::ReadGuard;

    /// Writes to the lock, blocking until able.
    fn write(&'a self) -> Self::WriteGuard;
}
/// The functions for [`RwLock`] that only work for sized types.
/// Separated to allow [`RwLock`] to be a trait object.
pub trait RwLockSized<'a>: Sized + RwLock<'a> + TryRwLockSized<'a> {
    /// Blocks until reading from the lock and then runs `func` on the result
    ///
    /// ## Implementation
    /// Should be overwritten by implementors if can be more optimal than
    /// creating a guard
    fn read_func<O>(&'a self, func: impl FnOnce(&Self::Item) -> O) -> O {
        func(self.read().deref())
    }

    /// Blocks until writing to the lock and then runs `func` on the result
    ///
    /// ## Implementation
    /// Should be overwritten by implementors if can be more optimal than
    /// creating a guard
    fn write_func<O>(&'a self, func: impl FnOnce(&mut Self::Item) -> O) -> O {
        func(self.write().deref_mut())
    }
}

/// A generic async reader-writer lock trait
///
/// ## Implementation
/// It is recommended to implement [`AsyncRwLockSized`] if the implement-ee can
/// be sized.
pub trait AsyncRwLock<'a>: TryRwLock<'a> {
    /// The read guard for async reading
    type AsyncReadGuard: Deref<Target = Self::Item>;
    /// The write guard for async writing
    type AsyncWriteGuard: DerefMut<Target = Self::Item>;
    /// The future returned by `read_async`
    type ReadFuture: Future<Output = Self::AsyncReadGuard>;
    /// The future returned by `write_async`
    type WriteFuture: Future<Output = Self::AsyncWriteGuard>;

    /// Reads the lock asynchronously, giving a future that will contain the
    /// read lock
    fn read_async(&'a self) -> Self::ReadFuture;

    /// Writes to the lock asynchronously, giving a future that will contain the
    /// write lock
    fn write_async(&'a self) -> Self::WriteFuture;
}

/// An rwlock that has read guards that can be upgraded
pub trait UpgradeRwLock<'a>: RwLock<'a>
where
    Self::ReadGuard: UpgradeReadGuard<'a, Item = Self::Item, WriteGuard = Self::WriteGuard>,
{
}
/// An async rwlock that has read guards that can be upgraded asynchronously
pub trait AsyncUpgradeRwLock<'a>: AsyncRwLock<'a>
where
    Self::AsyncReadGuard:
        AsyncUpgradeReadGuard<'a, Item = Self::Item, AsyncWriteGuard = Self::AsyncWriteGuard>,
{
}

/// A read guard that can be upgraded to a write guard
pub trait UpgradeReadGuard<'a>: Sized {
    /// Item guarded by this guard
    type Item: ?Sized;
    /// The write guard that this is upgraded to
    type WriteGuard: DerefMut<Target = Self::Item>;

    /// Upgrades this read guard into a write guard, blocking until done
    fn upgrade(self) -> Self::WriteGuard;
    /// Tries to upgrade this guard, returning `Err` if cannot immediately
    fn try_upgrade(self) -> Result<Self::WriteGuard, Self>;
}

/// A read guard that can be upgraded to a write guard asynchronously
pub trait AsyncUpgradeReadGuard<'a>: Sized {
    /// Item guarded by this guard
    type Item: ?Sized;
    /// The write guard this upgrades to
    type AsyncWriteGuard: DerefMut<Target = Self::Item>;
    /// The future returned by `upgrade_async`
    type UpgradeFuture: Future<Output = Self::AsyncWriteGuard>;

    /// Upgrades this guard asynchronously, returning a future that will contain
    /// the upgraded guard.
    fn upgrade_async(self) -> Self::UpgradeFuture;
}

/// An RwLock that can be timed out on
///
/// ## Implementation
/// It is recommended to implement [`TimeoutRwLockSized`] if the implement-ee
/// can be sized.
pub trait TimeoutRwLock<'a>: RwLock<'a> {
    /// Reads from the lock with a timeout
    fn read_timeout(&'a self, timeout: Duration) -> Option<Self::ReadGuard>;
    /// Writes to the lock with a timeout
    fn write_timeout(&'a self, timeout: Duration) -> Option<Self::WriteGuard>;
}
/// The functions for [`TimeoutRwLock`] that only work for sized types.
/// Separated to allow [`TimeoutRwLock`] to be a trait object.
pub trait TimeoutRwLockSized<'a>: Sized + TimeoutRwLock<'a> + RwLockSized<'a> {
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

/// An async RwLock that can be timed out on
///
/// ## Implementation
/// It is recommended to implement [`AsyncTimeoutRwLockSized`] if the
/// implement-ee can be sized.
pub trait AsyncTimeoutRwLock<'a>: AsyncRwLock<'a> {
    /// The future returned by [`AsyncTimeoutRwLock::read_timeout_async`]
    type ReadTimeoutFuture: Future<Output = Option<Self::AsyncReadGuard>>;
    /// The future returned by [`AsyncTimeoutRwLock::write_timeout_async`]
    type WriteTimeoutFuture: Future<Output = Option<Self::AsyncWriteGuard>>;

    /// Reads from the lock with a timeout asynchronously
    fn read_timeout_async(&'a self, timeout: Duration) -> Self::ReadTimeoutFuture;

    /// Writes to the lock with a timeout asynchronously
    fn write_timeout_async(&'a self, timeout: Duration) -> Self::WriteTimeoutFuture;
}

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
/// A raw rw lock that stores no data
pub unsafe trait RawRwLock: RawTryRwLock {
    /// Blocks until a reader is added to this lock
    fn add_reader(&self);
    /// Blocks until a writer is added to this lock
    fn add_writer(&self);
}
/// A raw timeout rw lock that stores no data
pub unsafe trait RawTimeoutRwLock: RawRwLock {
    /// Adds a reader to this lock with a timeout. Returns true if successful
    fn add_reader_timeout(&self, timeout: Duration) -> bool;
    /// Adds a writer to this lock with a timeout. Returns true if successful
    fn add_writer_timeout(&self, timeout: Duration) -> bool;
}
/// A raw async rw lock that stores no data
pub unsafe trait RawAsyncRwLock: RawTryRwLock {
    /// The future returned by [`RawAsyncRwLock::add_reader_async`]
    type AddReaderFuture: Future<Output = ()>;
    /// The future returned by [`RawAsyncRwLock::add_writer_async`]
    type AddWriterFuture: Future<Output = ()>;
    /// Adds a reader to the lock asynchronously
    fn add_reader_async(&self) -> Self::AddReaderFuture;
    /// Adds a writer to the lock asynchronously
    fn add_writer_async(&self) -> Self::AddWriterFuture;
}
/// A raw async timeout rw lock that stores no data
pub unsafe trait RawAsyncTimeoutRwLock: RawAsyncRwLock {
    /// The future returned by
    /// [`RawAsyncTimeoutRwLock::add_reader_timeout_async`]
    type AddReaderTimeoutFuture: Future<Output = bool>;
    /// The future returned by
    /// [`RawAsyncTimeoutRwLock::add_writer_timeout_async`]
    type AddWriterTimeoutFuture: Future<Output = bool>;
    /// Adds a reader to this lock with a timeout asynchronously. Returns true
    /// if successful.
    fn add_reader_timeout_async(&self, timeout: Duration) -> Self::AddReaderTimeoutFuture;
    /// Adds a writer to this lock with a timeout asynchronously. Returns true
    /// if successful.
    fn add_writer_timeout_async(&self, timeout: Duration) -> Self::AddWriterTimeoutFuture;
}

/// A custom rw lock that can be built from any [`RawTryRwLock`] variant
#[derive(Debug)]
pub struct CustomRwLock<T, R> {
    data: UnsafeCell<T>,
    raw_lock: R,
}
impl<T, R> CustomRwLock<T, R> {
    /// Creates a lock from a [`RawTryRwLock`] variant
    pub fn from_raw(raw_lock: R, data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            raw_lock,
        }
    }

    /// Creates a lock using thr [`RawTryRwLock`] variant's default
    /// implementation
    pub fn new(data: T) -> Self
    where
        R: Default,
    {
        Self::from_raw(R::default(), data)
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
impl<'a, T, R> RwLock<'a> for CustomRwLock<T, R>
where
    T: 'a,
    R: RawRwLock + 'a,
{
    fn read(&'a self) -> Self::ReadGuard {
        self.raw_lock.add_reader();
        CustomReadGuard { lock: self }
    }

    fn write(&'a self) -> Self::WriteGuard {
        self.raw_lock.add_writer();
        CustomWriteGuard { lock: self }
    }
}
impl<'a, T, R> RwLockSized<'a> for CustomRwLock<T, R>
where
    T: 'a,
    R: RawRwLock + 'a,
{
    fn read_func<O>(&'a self, func: impl FnOnce(&Self::Item) -> O) -> O {
        self.raw_lock.add_reader();
        let out = func(unsafe { &*self.data.get() });
        unsafe { self.raw_lock.remove_reader() }
        out
    }

    fn write_func<O>(&'a self, func: impl FnOnce(&mut Self::Item) -> O) -> O {
        self.raw_lock.add_writer();
        let out = func(unsafe { &mut *self.data.get() });
        unsafe { self.raw_lock.remove_writer() }
        out
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

/// The read guard for [`CustomRwLock`]
#[derive(Debug)]
pub struct CustomReadGuard<'a, T, R>
where
    R: RawTryRwLock,
{
    lock: &'a CustomRwLock<T, R>,
}
impl<'a, T, R> Deref for CustomReadGuard<'a, T, R>
where
    R: RawTryRwLock,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}
impl<'a, T, R> Drop for CustomReadGuard<'a, T, R>
where
    R: RawTryRwLock,
{
    fn drop(&mut self) {
        unsafe { self.lock.raw_lock.remove_reader() }
    }
}

/// The write guard for [`CustomRwLock`]
#[derive(Debug)]
pub struct CustomWriteGuard<'a, T, R>
where
    R: RawTryRwLock,
{
    lock: &'a CustomRwLock<T, R>,
}
impl<'a, T, R> Deref for CustomWriteGuard<'a, T, R>
where
    R: RawTryRwLock,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}
impl<'a, T, R> DerefMut for CustomWriteGuard<'a, T, R>
where
    R: RawTryRwLock,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}
impl<'a, T, R> Drop for CustomWriteGuard<'a, T, R>
where
    R: RawTryRwLock,
{
    fn drop(&mut self) {
        unsafe { self.lock.raw_lock.remove_writer() }
    }
}
