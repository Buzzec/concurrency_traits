#[cfg(feature = "alloc")]
use crate::queue::AsyncQueue;
use crate::queue::{Queue, TryQueue};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use async_trait::async_trait;

/// A queue that can try to be peeked into
pub trait TryPeekQueue: TryQueue {
    /// The type that `peek` returns
    type Peeked;
    /// Non blocking `peek`
    fn try_peek(&self) -> Option<Self::Peeked>;
}
/// A queue that can be peeked into
pub trait PeekQueue: Queue + TryPeekQueue {
    /// Peeks into the queue blocking until item is in
    fn peek(&self) -> Self::Peeked;
}
/// An async queue that can be peeked into
#[cfg(feature = "alloc")]
#[async_trait]
pub trait AsyncPeekQueue: AsyncQueue + TryPeekQueue {
    /// Peeks into the queue asynchronously
    async fn peek_async(&self) -> Self::Peeked;
}
