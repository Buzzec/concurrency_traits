use crate::rw_lock::{CustomReadGuard, CustomRwLock, CustomWriteGuard, RawTryRwLock, TryRwLock};
use alloc::boxed::Box;
use async_trait::async_trait;

/// A raw async rw lock that stores no data
#[async_trait]
pub unsafe trait RawAsyncRwLock: RawTryRwLock {
    /// Adds a reader to the lock asynchronously
    async fn add_reader_async(&self);
    /// Adds a writer to the lock asynchronously
    async fn add_writer_async(&self);
}
/// A generic async reader-writer lock trait
#[async_trait]
pub trait AsyncRwLock<'a>: TryRwLock<'a> {
    /// Reads the lock asynchronously, giving a future that will contain the
    /// read lock
    async fn read_async(&'a self) -> Self::ReadGuard;

    /// Writes to the lock asynchronously, giving a future that will contain the
    /// write lock
    async fn write_async(&'a self) -> Self::WriteGuard;
}

#[async_trait]
impl<'a, T, R> AsyncRwLock<'a> for CustomRwLock<T, R>
where
    T: 'a + Send + Sync,
    R: RawAsyncRwLock + 'a + Send + Sync,
{
    async fn read_async(&'a self) -> Self::ReadGuard {
        self.raw_lock.add_reader_async().await;
        CustomReadGuard { lock: self }
    }

    async fn write_async(&'a self) -> Self::WriteGuard {
        self.raw_lock.add_writer_async().await;
        CustomWriteGuard { lock: self }
    }
}
