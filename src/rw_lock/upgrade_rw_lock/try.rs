use crate::rw_lock::{CustomReadGuard, CustomRwLock, CustomWriteGuard, RawTryRwLock, TryRwLock};
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};

/// A raw rw lock which has guards that can try to be upgraded.
pub unsafe trait RawTryUpgradeRwLock: RawTryRwLock {
    /// Tries to upgrade a reader to a writer.
    ///
    /// # Safety
    /// Caller must ensure that a reader exists
    unsafe fn try_upgrade(&self) -> bool;
}
/// An rwlock that has read guards that can try to be upgraded
pub trait TryUpgradeRwLock<'a>: TryRwLock<'a>
where
    Self::ReadGuard: TryUpgradeReadGuard<'a, Item = Self::Item, WriteGuard = Self::WriteGuard>,
{
}
/// A read guard that can be try to be upgraded to a write guard
pub trait TryUpgradeReadGuard<'a>: Sized + Deref<Target = Self::Item> {
    /// Item guarded by this guard
    type Item: ?Sized;
    /// The write guard that this is upgraded to
    type WriteGuard: DerefMut<Target = Self::Item>;

    /// Tries to upgrade this guard, returning `Err` if cannot immediately
    fn try_upgrade(self) -> Result<Self::WriteGuard, Self>;
}

impl<'a, T, R> TryUpgradeRwLock<'a> for CustomRwLock<T, R>
where
    T: 'a,
    R: RawTryUpgradeRwLock + 'a,
{
}

impl<'a, T, R> TryUpgradeReadGuard<'a> for CustomReadGuard<'a, T, R>
where
    R: RawTryUpgradeRwLock,
{
    type Item = T;
    type WriteGuard = CustomWriteGuard<'a, T, R>;

    fn try_upgrade(self) -> Result<Self::WriteGuard, Self> {
        match unsafe { self.lock.raw_lock.try_upgrade() } {
            true => {
                let self_manual = ManuallyDrop::new(self);
                Ok(CustomWriteGuard {
                    lock: self_manual.lock,
                })
            }
            false => Err(self),
        }
    }
}
