/// A generic queue that supports try operations
pub trait TryQueue {
    /// The type the queue holds.
    type Item;
    /// Tries to append an item to the queue returning `None` if unsuccessful
    fn try_push(&self, value: Self::Item) -> Result<(), Self::Item>;
    /// Tries to receive an item from the queue returning `None` if none
    /// available
    fn try_pop(&self) -> Option<Self::Item>;
}
