#[cfg(feature = "alloc")]
use crate::queue::{AsyncPeekQueue, AsyncQueue};
use crate::queue::{PeekQueue, Queue, TryPeekQueue, TryQueue};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use async_trait::async_trait;

/// A queue that can try to be read in reverse.
pub trait TryReverseQueue: TryQueue {
    /// Non blocking version of `receive_back`
    fn try_pop_back(&self) -> Option<Self::Item>;
}
/// A queue that can be read in reverse.
pub trait ReverseQueue: TryReverseQueue + Queue {
    /// Reads from the back of the queue
    fn pop_back(&self) -> Self::Item;
}
/// An asynchronous queue that can be read in reverse
#[cfg(feature = "alloc")]
#[async_trait]
pub trait AsyncReverseQueue: AsyncQueue {
    /// Reads the back of the queue
    async fn pop_back_async(&self) -> Self::Item;
}

/// A queue that can try to be peeked from behind
pub trait TryPeekReverseQueue: TryPeekQueue + TryReverseQueue {
    /// Peeks the rear item without blocking
    fn try_peek_back(&self) -> Option<Self::Peeked>;
}
/// A queue that can be peeked from behind
pub trait PeekReverseQueue: PeekQueue + ReverseQueue + TryPeekReverseQueue {
    /// Peeks the rear item of the queue blocking until available
    fn peek_back(&self) -> Self::Peeked;
}
/// A queue that can be peeked from behind asynchronously
#[cfg(feature = "alloc")]
#[async_trait]
pub trait AsyncPeekReverseQueue: AsyncPeekQueue + AsyncReverseQueue {
    /// Peeks the rear item of the queue blocking until available
    async fn peek_back_async(&self) -> Self::Peeked;
}
