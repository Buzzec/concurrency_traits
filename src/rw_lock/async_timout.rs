use crate::rw_lock::{CustomReadGuard, CustomRwLock, CustomWriteGuard, RawTryRwLock, TryRwLock};
use alloc::boxed::Box;
use async_trait::async_trait;
use core::time::Duration;

/// A raw async timeout rw lock that stores no data
#[async_trait]
pub unsafe trait RawAsyncTimeoutRwLock: RawTryRwLock {
    /// Adds a reader to this lock with a timeout asynchronously. Returns true
    /// if successful.
    async fn add_reader_timeout_async(&self, timeout: Duration) -> bool;
    /// Adds a writer to this lock with a timeout asynchronously. Returns true
    /// if successful.
    async fn add_writer_timeout_async(&self, timeout: Duration) -> bool;
}
/// An async RwLock that can be timed out on
#[async_trait]
pub trait AsyncTimeoutRwLock<'a>: TryRwLock<'a> {
    /// Reads from the lock with a timeout asynchronously
    async fn read_timeout_async(&'a self, timeout: Duration) -> Option<Self::ReadGuard>;

    /// Writes to the lock with a timeout asynchronously
    async fn write_timeout_async(&'a self, timeout: Duration) -> Option<Self::WriteGuard>;
}

#[async_trait]
impl<'a, T, R> AsyncTimeoutRwLock<'a> for CustomRwLock<T, R>
where
    T: 'a + Send + Sync,
    R: RawAsyncTimeoutRwLock + 'a + Send + Sync,
{
    async fn read_timeout_async(&'a self, timeout: Duration) -> Option<Self::ReadGuard> {
        match self.raw_lock.add_reader_timeout_async(timeout).await {
            true => Some(CustomReadGuard { lock: self }),
            false => None,
        }
    }

    async fn write_timeout_async(&'a self, timeout: Duration) -> Option<Self::WriteGuard> {
        match self.raw_lock.add_writer_timeout_async(timeout).await {
            true => Some(CustomWriteGuard { lock: self }),
            false => None,
        }
    }
}
