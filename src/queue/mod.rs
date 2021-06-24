//! Generic implementations for queues.

mod impls;
pub use impls::*;

#[cfg(feature = "alloc")]
mod r#async;
#[cfg(feature = "alloc")]
pub use r#async::*;

#[cfg(feature = "alloc")]
mod async_timeout;
#[cfg(feature = "alloc")]
pub use async_timeout::*;

mod double_ended;
pub use double_ended::*;

mod length_queue;
pub use length_queue::*;

mod peek;
pub use peek::*;

mod prepend;
pub use prepend::*;

mod reverse;
pub use reverse::*;

mod timeout;
pub use timeout::*;

mod r#try;
pub use r#try::*;

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
        assert_eq!(queue.try_pop(), Some(200));
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
        assert_eq!(queue.pop(), 200);
        assert!(queue.try_pop().is_none());
    }
}
