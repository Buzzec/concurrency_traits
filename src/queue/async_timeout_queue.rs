use crate::queue::AsyncQueue;
use core::time::Duration;
use async_trait::async_trait;
use alloc::boxed::Box;

/// An Async Queue that can timeout on push and pop operations
#[async_trait]
pub trait AsyncTimeoutQueue: AsyncQueue {
    /// Pushes an item to the queue asynchronously with a timeout
    async fn push_timeout_async(&self, value: Self::Item, timeout: Duration) -> Result<(), Self::Item>;
    /// Pops an item from the queue asynchronously with a timeout
    async fn pop_timeout_async(&self, timeout: Duration) -> Option<Self::Item>;
}
