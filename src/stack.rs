//! Traits for a FIFO stack.

use core::future::Future;

/// A Stack with try operations
pub trait TryStack {
    /// The item this stack stores
    type Item;

    /// Tries to push an item onto the stack
    fn try_push(&self, value: Self::Item) -> Result<(), Self::Item>;
    /// Tries to pop an item off the stack
    fn try_pop(&self) -> Option<Self::Item>;
}
/// A stack with pop and push
pub trait Stack: TryStack {
    /// Pushes an item onto the stack
    fn push(&self, value: Self::Item);

    /// Pops an item off the stack
    fn pop(&self) -> Self::Item;
}
/// A stack with async operations
pub trait AsyncStack: TryStack {
    /// The future returned by `push_async`
    type PushFuture: Future<Output = ()>;
    /// The future returned by `pop_async`
    type PopFuture: Future<Output = Self::Item>;

    /// Push an item onto the stack asynchronously
    fn push_async(&self, value: Self::Item) -> Self::PushFuture;
    /// Pops an item from the stack asynchronously
    fn pop_async(&self) -> Self::PopFuture;
}
