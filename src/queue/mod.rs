//! Generic implementations for queues.

mod impls;
pub use impls::*;

#[cfg(feature = "alloc")]
mod async_queue;
#[cfg(feature = "alloc")]
pub use async_queue::*;

#[cfg(feature = "alloc")]
mod async_timeout_queue;
#[cfg(feature = "alloc")]
pub use async_timeout_queue::*;

mod double_ended_queue;
pub use double_ended_queue::*;

mod peek_queue;
pub use peek_queue::*;

mod prepend_queue;
pub use prepend_queue::*;

mod reverse_queue;
pub use reverse_queue::*;

mod timeout_queue;
pub use timeout_queue::*;

mod try_queue;
pub use try_queue::*;

/// A generic queue that can push and pop in FIFO order
pub trait Queue: TryQueue {
    /// Appends an item to the end of the queue blocking until appended
    fn push(&self, value: Self::Item);

    /// Blocks until an item is received from the queue
    fn pop(&self) -> Self::Item;
}

#[cfg(test)]
pub(super) mod test {
    use crate::queue::{Queue, TryQueue};
    pub fn try_queue_test<Q>(queue: Q)
    where
        Q: TryQueue<Item = usize>,
    {
        assert!(queue.try_pop().is_none());
        assert!(queue.try_push(100).is_ok());
        assert_eq!(queue.try_pop(), Some(100));
        assert!(queue.try_push(200).is_ok());
        queue.clear();
        assert!(queue.try_pop().is_none());
    }

    pub fn queue_test<Q>(queue: Q)
    where
        Q: Queue<Item = usize>,
    {
        assert!(queue.try_pop().is_none());
        queue.push(100);
        assert_eq!(queue.pop(), 100);
        queue.push(200);
        queue.clear();
        assert!(queue.try_pop().is_none());
    }
}
