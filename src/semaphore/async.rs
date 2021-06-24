use crate::semaphore::TrySemaphore;
use alloc::boxed::Box;
use async_trait::async_trait;

/// A generic semaphore that can be waited on asynchronously.
///
/// # Safety
/// This trait is marked as unsafe to allow for safe code to rely on the
/// standard semaphore contract. [`Default`] implementations should initialize
/// the count at 0.
#[async_trait]
pub unsafe trait AsyncSemaphore: TrySemaphore {
    /// Awaits until able to decrement then decrements.
    async fn wait_async(&self);
}
