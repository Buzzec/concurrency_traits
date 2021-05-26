use crate::mutex::{RawAsyncMutex, AsyncMutex, CustomMutex, CustomMutexGuard};
use core::time::Duration;
use async_trait::async_trait;
use alloc::boxed::Box;

/// A raw async mutex that can be timed out and holds no data.
#[async_trait]
pub unsafe trait RawAsyncTimeoutMutex: RawAsyncMutex {
    /// Locks the mutex on a timeout asynchronously. Returns true if locked.
    async fn lock_timeout_async(&self, timeout: Duration) -> bool;
}
/// An async mutex that locking can timeout on.
#[async_trait]
pub trait AsyncTimeoutMutex<'a>: AsyncMutex<'a> {
    /// Locks the mutex asynchronously with a timeout.
    async fn lock_timeout_async(&'a self, timeout: Duration) -> Option<Self::Guard>;
}

#[async_trait]
impl<'a, T, M> AsyncTimeoutMutex<'a> for CustomMutex<T, M>
where
    T: 'a + Send,
    M: RawAsyncTimeoutMutex + 'a + Send + Sync,
{
    async fn lock_timeout_async(&'a self, timeout: Duration) -> Option<Self::Guard> {
        match self.raw_mutex.lock_timeout_async(timeout).await {
            true => Some(CustomMutexGuard { mutex: self }),
            false => None,
        }
    }
}
