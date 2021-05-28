use crate::rw_lock::{
    CustomReadGuard, CustomRwLock, CustomWriteGuard, RawTryUpgradeRwLock, TryUpgradeReadGuard,
    TryUpgradeRwLock,
};
use alloc::boxed::Box;
use async_trait::async_trait;
use core::mem::ManuallyDrop;

/// Raw version of [`AsyncUpgradeRwLock`].
#[async_trait]
pub unsafe trait RawAsyncUpgradeRwLock: RawTryUpgradeRwLock {
    /// Upgrades a reader to a writer asynchronously.
    ///
    /// # Safey
    /// Caller must ensure a reader exists.
    async unsafe fn upgrade_async(&self);
}
/// An async rwlock that has read guards that can be upgraded asynchronously
pub trait AsyncUpgradeRwLock<'a>: TryUpgradeRwLock<'a>
where
    Self::ReadGuard: AsyncUpgradeReadGuard<'a, Item = Self::Item, WriteGuard = Self::WriteGuard>,
{
}
/// A read guard that can be upgraded to a write guard asynchronously
#[async_trait]
pub trait AsyncUpgradeReadGuard<'a>: Sized + TryUpgradeReadGuard<'a> {
    /// Upgrades this guard asynchronously, returning a future that will contain
    /// the upgraded guard.
    async fn upgrade_async(self) -> Self::WriteGuard;
}

impl<'a, T, R> AsyncUpgradeRwLock<'a> for CustomRwLock<T, R>
where
    T: 'a + Send + Sync,
    R: RawAsyncUpgradeRwLock + 'a + Send + Sync,
{
}
#[async_trait]
impl<'a, T, R> AsyncUpgradeReadGuard<'a> for CustomReadGuard<'a, T, R>
where
    T: 'a + Send + Sync,
    R: RawAsyncUpgradeRwLock + 'a + Send + Sync,
{
    async fn upgrade_async(self) -> Self::WriteGuard {
        unsafe {
            self.lock.raw_lock.upgrade_async().await;
        }
        let self_manual = ManuallyDrop::new(self);
        CustomWriteGuard {
            lock: self_manual.lock,
        }
    }
}
