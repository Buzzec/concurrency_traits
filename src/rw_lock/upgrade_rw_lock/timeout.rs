use crate::rw_lock::{
    CustomReadGuard, CustomRwLock, CustomWriteGuard, RawTryUpgradeRwLock, TryUpgradeReadGuard,
    TryUpgradeRwLock,
};
use core::mem::ManuallyDrop;
use core::time::Duration;

/// A raw rw lock which has guards that can be upgraded on a timeout.
pub unsafe trait RawUpgradeTimeoutRwLock: RawTryUpgradeRwLock {
    /// Blocks until lock is changed from read to write (true) or times out
    /// (false).
    ///
    /// # Safety
    /// Caller must ensure that a reader exists.
    unsafe fn upgrade_timeout(&self, timeout: Duration) -> bool;
}
/// An rwlock that has read guards that can be upgraded on a timeout.
pub trait UpgradeTimeoutRwLock<'a>: TryUpgradeRwLock<'a>
where
    Self::ReadGuard: UpgradeTimeoutReadGuard<'a, Item = Self::Item, WriteGuard = Self::WriteGuard>,
{
}
/// A read guard that can be upgraded to a write guard
pub trait UpgradeTimeoutReadGuard<'a>: TryUpgradeReadGuard<'a> {
    /// Upgrades this read guard into a write guard, blocking until done
    fn upgrade_timeout(self, timeout: Duration) -> Result<Self::WriteGuard, Self>;
}

impl<'a, T, R> UpgradeTimeoutRwLock<'a> for CustomRwLock<T, R>
where
    T: 'a,
    R: RawUpgradeTimeoutRwLock + 'a,
{
}

impl<'a, T, R> UpgradeTimeoutReadGuard<'a> for CustomReadGuard<'a, T, R>
where
    R: RawUpgradeTimeoutRwLock,
{
    fn upgrade_timeout(self, timeout: Duration) -> Result<Self::WriteGuard, Self> {
        match unsafe { self.lock.raw_lock.upgrade_timeout(timeout) } {
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
