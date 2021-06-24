#[cfg(feature = "alloc")]
use crate::queue::AsyncQueue;
use crate::queue::{AsyncTimeoutQueue, Queue, TimeoutQueue, TryQueue};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use async_trait::async_trait;
use core::time::Duration;

/// A queue that can be attempt to be prepended to
pub trait TryPrependQueue: TryQueue {
    /// Adds an item to the front of the queue without blocking
    fn try_push_front(&self, value: Self::Item) -> Result<(), Self::Item>;
}
/// A queue that can be prepended (items placed in front)
pub trait PrependQueue: Queue + TryPrependQueue {
    /// Adds an item to the front of the queue blocking until able
    fn push_front(&self, value: Self::Item);
}
/// A queue that can be prepended (items placed in front) with a timeout
pub trait PrependTimeoutQueue: TryPrependQueue + TimeoutQueue {
    /// Adds an item to the front of the queue blocking until able or timing
    /// out.
    fn push_front_timeout(&self, value: Self::Item, timeout: Duration) -> Result<(), Self::Item>;
}
/// An async queue that can be prepended (items placed in front)
#[cfg(feature = "alloc")]
#[async_trait]
pub trait AsyncPrependQueue: TryPrependQueue + AsyncQueue {
    /// Adds to the front of the queue asynchronously
    async fn push_front_async(&self, value: Self::Item);
}
/// An async queue that can be prepended (items placed in front) with a timeout
#[cfg(feature = "alloc")]
#[async_trait]
pub trait AsyncPrependTimeoutQueue: TryPrependQueue + AsyncTimeoutQueue {
    /// Adds to the front of the queue asynchronously with a timeout
    async fn push_front_timeout_async(
        &self,
        value: Self::Item,
        timeout: Duration,
    ) -> Result<(), Self::Item>;
}
