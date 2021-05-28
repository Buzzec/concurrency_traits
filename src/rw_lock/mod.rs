//! Traits for RwLocks

mod impls;
pub use impls::*;

#[cfg(feature = "alloc")]
mod r#async;
#[cfg(feature = "alloc")]
pub use r#async::*;

#[cfg(feature = "alloc")]
mod async_timout;
#[cfg(feature = "alloc")]
pub use async_timout::*;

mod custom;
pub use custom::*;

mod timeout;
pub use timeout::*;

mod r#try;
pub use r#try::*;

mod upgrade_rw_lock;
pub use upgrade_rw_lock::*;

use core::ops::{Deref, DerefMut};

/// A raw rw lock that stores no data
pub unsafe trait RawRwLock: RawTryRwLock {
    /// Blocks until a reader is added to this lock
    fn add_reader(&self);
    /// Blocks until a writer is added to this lock
    fn add_writer(&self);
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
