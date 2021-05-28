#[cfg(feature = "alloc")]
mod r#async;
#[cfg(feature = "alloc")]
pub use r#async::*;

#[cfg(feature = "alloc")]
mod async_timeout;
#[cfg(feature = "alloc")]
pub use async_timeout::*;

mod timeout;
pub use timeout::*;

mod r#try;
pub use r#try::*;

use crate::rw_lock::{CustomReadGuard, CustomRwLock, CustomWriteGuard, RawTryRwLock, TryRwLock};
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};

/// A raw rw lock which has guards that can be upgraded.
pub unsafe trait RawUpgradeRwLock: RawTryUpgradeRwLock {
    /// Blocks until lock is changed from read to write.
    ///
    /// # Safety
    /// Caller must ensure that a reader exists.
    unsafe fn upgrade(&self);
}
/// A raw rw lock which has guards that can be downgraded.
pub unsafe trait RawDowngradeRwLock: RawTryRwLock {
    /// Changes lock from writing to 1 writer.
    ///
    /// # Safety
    /// Caller must ensure that a writer exists.
    unsafe fn downgrade(&self);
}
/// An rwlock that has read guards that can be upgraded
pub trait UpgradeRwLock<'a>: TryUpgradeRwLock<'a>
where
    Self::ReadGuard: UpgradeReadGuard<'a, Item = Self::Item, WriteGuard = Self::WriteGuard>,
{
}

/// An rwlock that has write guards that can be downgraded.
pub trait DowngradeRwLock<'a>: TryRwLock<'a>
where
    Self::WriteGuard: DowngradeWriteGuard<'a, Item = Self::Item, ReadGuard = Self::ReadGuard>,
{
}

/// A read guard that can be upgraded to a write guard
pub trait UpgradeReadGuard<'a>: TryUpgradeReadGuard<'a> {
    /// Upgrades this read guard into a write guard, blocking until done
    fn upgrade(self) -> Self::WriteGuard;
}

/// A write guard that can be downgraded.
pub trait DowngradeWriteGuard<'a>: Sized + DerefMut<Target = Self::Item> {
    /// Item guarded by this guard
    type Item: ?Sized;
    /// The write guard that this is upgraded to
    type ReadGuard: Deref<Target = Self::Item>;

    /// Downgrades this write guard into a read guard, blocking until done
    fn downgrade(self) -> Self::ReadGuard;
}

impl<'a, T, R> UpgradeRwLock<'a> for CustomRwLock<T, R>
where
    T: 'a,
    R: RawUpgradeRwLock + 'a,
{
}
impl<'a, T, R> DowngradeRwLock<'a> for CustomRwLock<T, R>
where
    T: 'a,
    R: RawDowngradeRwLock + 'a,
{
}

impl<'a, T, R> UpgradeReadGuard<'a> for CustomReadGuard<'a, T, R>
where
    R: RawUpgradeRwLock,
{
    fn upgrade(self) -> Self::WriteGuard {
        unsafe { self.lock.raw_lock.upgrade() }
        let self_manual = ManuallyDrop::new(self);
        CustomWriteGuard {
            lock: self_manual.lock,
        }
    }
}

impl<'a, T, R> DowngradeWriteGuard<'a> for CustomWriteGuard<'a, T, R>
where
    R: RawDowngradeRwLock,
{
    type Item = T;
    type ReadGuard = CustomReadGuard<'a, T, R>;

    fn downgrade(self) -> Self::ReadGuard {
        unsafe { self.lock.raw_lock.downgrade() }
        let self_manual = ManuallyDrop::new(self);
        CustomReadGuard {
            lock: self_manual.lock,
        }
    }
}
