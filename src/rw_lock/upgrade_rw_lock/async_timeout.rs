use crate::rw_lock::{
    CustomReadGuard, CustomRwLock, CustomWriteGuard, RawTryUpgradeRwLock, TryUpgradeReadGuard,
    TryUpgradeRwLock,
};
use alloc::boxed::Box;
use async_trait::async_trait;
use core::mem::ManuallyDrop;
use core::time::Duration;

/// Raw version of [`AsyncTimeoutUpgradeRwLock`].
#[async_trait]
pub unsafe trait RawAsyncTimeoutUpgradeRwLock: RawTryUpgradeRwLock {
    /// Upgrades a reader to a writer asynchronously with a timeout. Returns [`true`] on success.
    ///
    /// # Safey
    /// Caller must ensure a reader exists.
    async unsafe fn upgrade_timeout_async(&self, timeout: Duration) -> bool;
}
/// An async rwlock that has read guards that can be upgraded asynchronously with a timeout.
pub trait AsyncTimeoutUpgradeRwLock<'a>: TryUpgradeRwLock<'a>
where
    Self::ReadGuard:
        AsyncTimoutUpgradeReadGuard<'a, Item = Self::Item, WriteGuard = Self::WriteGuard>,
{
}

/// A read guard that can be upgraded to a write guard asynchronously with a timeout.
#[cfg(feature = "alloc")]
#[async_trait]
pub trait AsyncTimoutUpgradeReadGuard<'a>: Sized + TryUpgradeReadGuard<'a> {
    /// Upgrades this guard asynchronously, returning a future that will contain
    /// the upgraded guard or times out.
    async fn upgrade_timeout_async(self, timeout: Duration) -> Result<Self::WriteGuard, Self>;
}

impl<'a, T, R> AsyncTimeoutUpgradeRwLock<'a> for CustomRwLock<T, R>
where
    T: 'a + Send + Sync,
    R: RawAsyncTimeoutUpgradeRwLock + 'a + Send + Sync,
{
}
#[async_trait]
impl<'a, T, R> AsyncTimoutUpgradeReadGuard<'a> for CustomReadGuard<'a, T, R>
where
    T: 'a + Send + Sync,
    R: RawAsyncTimeoutUpgradeRwLock + 'a + Send + Sync,
{
    async fn upgrade_timeout_async(self, timeout: Duration) -> Result<Self::WriteGuard, Self> {
        match unsafe { self.lock.raw_lock.upgrade_timeout_async(timeout).await } {
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
