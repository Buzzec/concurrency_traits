use crate::queue::TryQueue;
use core::time::Duration;

/// A Queue that can timeout on push and pop operations
pub trait TimeoutQueue: TryQueue {
    /// Appends an item to the end of the queue blocking until appended or
    /// timeout
    fn push_timeout(&self, value: Self::Item, timeout: Duration) -> Result<(), Self::Item>;
    /// Blocks until an item is received from the queue or timeout
    fn pop_timeout(&self, timeout: Duration) -> Option<Self::Item>;
}
