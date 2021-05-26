use crate::mutex::{TryMutex, RawTryMutex, CustomMutex, CustomMutexGuard};
use async_trait::async_trait;
use alloc::boxed::Box;

/// A raw async mutex that hold no data but the lock itself.
#[async_trait]
pub unsafe trait RawAsyncMutex: RawTryMutex {
    /// Locks the mutex asynchronously
    async fn lock_async(&self);
}
/// A generic async mutex trait
#[async_trait]
pub trait AsyncMutex<'a>: TryMutex<'a> {
    /// Locks the mutex asynchronously, returning a future with the guard.
    async fn lock_async(&'a self) -> Self::Guard;
}

#[async_trait]
impl<'a, T, M> AsyncMutex<'a> for CustomMutex<T, M>
where
    T: 'a + Send,
    M: RawAsyncMutex + 'a + Send + Sync,
{
    async fn lock_async(&'a self) -> Self::Guard {
        self.raw_mutex.lock_async().await;
        CustomMutexGuard { mutex: self }
    }
}
