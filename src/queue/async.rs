use crate::queue::TryQueue;
use alloc::boxed::Box;
use async_trait::async_trait;

/// A Queue that can be accessed asynchronously
#[async_trait]
pub trait AsyncQueue: TryQueue {
    /// Appends to the queue asynchronously.
    async fn push_async(&self, value: Self::Item);
    /// Receives from the queue asynchronously.
    async fn pop_async(&self) -> Self::Item;
}
