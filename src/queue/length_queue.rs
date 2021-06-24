use crate::queue::TryQueue;

/// A Queue who's length can be read.
pub trait LengthQueue: TryQueue {
    /// The current length of the queue.
    fn len(&self) -> usize;
    /// Returns true if the queue is empty.
    fn is_empty(&self) -> bool;
}
