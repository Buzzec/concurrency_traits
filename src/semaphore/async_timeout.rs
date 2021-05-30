use crate::semaphore::TrySemaphore;
use alloc::boxed::Box;
use async_trait::async_trait;
use core::time::Duration;

/// A generic semaphore that can be waited on asynchronously with a timeout.
///
/// # Safety
/// This trait is marked as unsafe to allow for safe code to rely on the standard semaphore contract.
/// [`Default`] implementations should initialize the count at 0.
#[async_trait]
pub unsafe trait AsyncTimeoutSemaphore: TrySemaphore {
    /// Awaits until timeout or able to decrement, returning true if decremented or false if timed out.
    #[must_use]
    async fn wait_timeout_async(&self, timeout: Duration) -> bool;
}
