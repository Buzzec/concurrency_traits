#[cfg(feature = "alloc")]
use crate::queue::AsyncQueue;
use crate::queue::{Queue, TryQueue};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use async_trait::async_trait;

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
/// An async queue that can be prepended (items placed in front)
#[cfg(feature = "alloc")]
#[async_trait]
pub trait AsyncPrependQueue: AsyncQueue {
    /// Adds to the front of the queue asynchronously
    async fn push_front_async(&self, value: Self::Item);
}
