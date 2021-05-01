//! Generic implementations for queues.

#[cfg(feature = "alloc")]
mod queue_alloc;

#[cfg(feature = "alloc")]
pub use queue_alloc::*;

use core::future::Future;

/// A generic queue that supports try operations
pub trait TryQueue {
    /// The type the queue holds.
    type Item;
    /// Tries to append an item to the queue returning `None` if unsuccessful
    fn try_push(&self, value: Self::Item) -> Result<(), Self::Item>;
    /// Tries to receive an item from the queue returning `None` if none
    /// available
    fn try_pop(&self) -> Option<Self::Item>;
    /// Clears the queue
    fn clear(&self);
}

/// A generic queue that can push and pop in FIFO order
pub trait Queue: TryQueue {
    /// Appends an item to the end of the queue blocking until appended
    fn push(&self, value: Self::Item);

    /// Blocks until an item is received from the queue
    fn pop(&self) -> Self::Item;
}
/// A Queue that can be accessed asynchronously
pub trait AsyncQueue {
    /// The type the queue holds.
    type AsyncItem;
    /// The future returned by `append_async`
    type PushFuture: Future<Output = ()>;
    /// The future returned by `receive_async`
    type PopFuture: Future<Output = Self::AsyncItem>;

    /// Appends to the queue asynchronously.
    fn push_async(&self, value: Self::AsyncItem) -> Self::PushFuture;
    /// Receives from the queue asynchronously.
    fn pop_async(&self) -> Self::PopFuture;
}
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
pub trait AsyncPrependQueue: AsyncQueue {
    /// The future returned by `prepend_async`
    type PushBackFuture: Future<Output = ()>;

    /// Adds to the front of the queue asynchronously
    fn push_front_async(&self, value: Self::AsyncItem) -> Self::PushBackFuture;
}
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
pub trait AsyncReverseQueue: AsyncQueue {
    /// The future returned by `receive_back_async`
    type PopBackFuture: Future<Output = Self::AsyncItem>;

    /// Reads the back of the queue
    fn pop_back_async(&self) -> Self::PopBackFuture;
}
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
pub trait AsyncPeekQueue: AsyncQueue {
    /// The type that is peeked
    type AsyncPeeked;
    /// The future returned by `peek_async`
    type PeekFuture: Future<Output = Self::AsyncPeeked>;

    /// Peeks into the queue asynchronously
    fn peek_async(&self) -> Self::PeekFuture;
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
pub trait AsyncPeekReverseQueue: AsyncPeekQueue + AsyncReverseQueue {
    /// The future returned by `peek_back_async`
    type PeekBackFuture: Future<Output = Self::AsyncPeeked>;
    /// Peeks the rear item of the queue blocking until available
    fn peek_back_async(&self) -> Self::PeekBackFuture;
}
/// A queue that can try to be written and read from both ends
pub trait TryDoubleEndedQueue: TryPrependQueue + TryReverseQueue {}
/// A queue that can be written and read from both ends
pub trait DoubleEndedQueue: PrependQueue + ReverseQueue + TryDoubleEndedQueue {}
/// An async queue that can be written and read from both ends
pub trait AsyncDoubleEndedQueue: AsyncPrependQueue + AsyncReverseQueue {}
