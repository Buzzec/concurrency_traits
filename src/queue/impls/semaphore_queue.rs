use crate::mutex::{Mutex, SpinLock};
use crate::queue::*;
use crate::semaphore::*;
use crate::ThreadFunctions;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use async_trait::async_trait;
use core::time::Duration;
use num::Zero;

/// A queue based on a semaphore to block on.
#[derive(Debug)]
pub struct SemaphoreQueue<T, S, CS> {
    queue: SpinLock<VecDeque<T>, CS>,
    semaphore: S,
}
impl<T, S, CS> SemaphoreQueue<T, S, CS>
where
    S: ReadoutSemaphore,
{
    /// Gets the length of the queue.
    pub fn len(&self) -> S::Count {
        self.semaphore.count()
    }

    /// Tells whether the queue is empty.
    pub fn is_empty(&self) -> bool
    where
        S::Count: Zero,
    {
        self.semaphore.count().is_zero()
    }
}
impl<T, S, CS> Default for SemaphoreQueue<T, S, CS>
where
    S: Default,
{
    fn default() -> Self {
        Self {
            queue: Default::default(),
            semaphore: S::default(),
        }
    }
}

impl<T, S, CS> TryQueue for SemaphoreQueue<T, S, CS>
where
    S: TrySemaphore,
    CS: ThreadFunctions,
{
    type Item = T;

    fn try_push(&self, value: Self::Item) -> Result<(), Self::Item> {
        self.queue.lock().push_back(value);
        self.semaphore.signal();
        Ok(())
    }

    fn try_pop(&self) -> Option<Self::Item> {
        match self.semaphore.try_wait() {
            true => Some(self.queue.lock().pop_front().unwrap()),
            false => None,
        }
    }
}
impl<T, S, CS> Queue for SemaphoreQueue<T, S, CS>
where
    S: Semaphore,
    CS: ThreadFunctions,
{
    fn push(&self, value: Self::Item) {
        self.try_push(value)
            .unwrap_or_else(|_| panic!("try_push failed!"));
    }

    fn pop(&self) -> Self::Item {
        self.semaphore.wait();
        self.queue.lock().pop_front().unwrap()
    }
}
#[async_trait]
impl<T, S, CS> AsyncQueue for SemaphoreQueue<T, S, CS>
where
    T: Send,
    S: AsyncSemaphore + Send + Sync,
    CS: ThreadFunctions,
{
    async fn push_async(&self, value: Self::Item) {
        self.try_push(value)
            .unwrap_or_else(|_| panic!("try_push failed!"))
    }

    async fn pop_async(&self) -> Self::Item {
        self.semaphore.wait_async().await;
        self.queue.lock().pop_back().unwrap()
    }
}

impl<T, S, CS> TryPrependQueue for SemaphoreQueue<T, S, CS>
where
    S: TrySemaphore,
    CS: ThreadFunctions,
{
    fn try_push_front(&self, value: Self::Item) -> Result<(), Self::Item> {
        self.queue.lock().push_front(value);
        self.semaphore.signal();
        Ok(())
    }
}
impl<T, S, CS> PrependQueue for SemaphoreQueue<T, S, CS>
where
    S: Semaphore,
    CS: ThreadFunctions,
{
    fn push_front(&self, value: Self::Item) {
        self.try_push_front(value)
            .unwrap_or_else(|_| panic!("try_push_front failed!"));
    }
}
#[async_trait]
impl<T, S, CS> AsyncPrependQueue for SemaphoreQueue<T, S, CS>
where
    T: Send,
    S: AsyncSemaphore + Send + Sync,
    CS: ThreadFunctions,
{
    async fn push_front_async(&self, value: Self::Item) {
        self.try_push_front(value)
            .unwrap_or_else(|_| panic!("try_push_front failed!"));
    }
}

impl<T, S, CS> TryReverseQueue for SemaphoreQueue<T, S, CS>
where
    S: TrySemaphore,
    CS: ThreadFunctions,
{
    fn try_pop_back(&self) -> Option<Self::Item> {
        match self.semaphore.try_wait() {
            true => Some(self.queue.lock().pop_back().unwrap()),
            false => None,
        }
    }
}
impl<T, S, CS> ReverseQueue for SemaphoreQueue<T, S, CS>
where
    S: Semaphore,
    CS: ThreadFunctions,
{
    fn pop_back(&self) -> Self::Item {
        self.try_pop_back().unwrap()
    }
}
#[async_trait]
impl<T, S, CS> AsyncReverseQueue for SemaphoreQueue<T, S, CS>
where
    T: Send,
    S: AsyncSemaphore + Send + Sync,
    CS: ThreadFunctions,
{
    async fn pop_back_async(&self) -> Self::Item {
        self.semaphore.wait_async().await;
        self.queue.lock().pop_back().unwrap()
    }
}

impl<T, S, CS> TryDoubleEndedQueue for SemaphoreQueue<T, S, CS>
where
    S: TrySemaphore,
    CS: ThreadFunctions,
{
}
impl<T, S, CS> DoubleEndedQueue for SemaphoreQueue<T, S, CS>
where
    S: Semaphore,
    CS: ThreadFunctions,
{
}
impl<T, S, CS> AsyncDoubleEndedQueue for SemaphoreQueue<T, S, CS>
where
    T: Send,
    S: AsyncSemaphore + Send + Sync,
    CS: ThreadFunctions,
{
}

impl<T, S, CS> TimeoutQueue for SemaphoreQueue<T, S, CS>
where
    S: TimeoutSemaphore,
    CS: ThreadFunctions,
{
    fn push_timeout(&self, value: Self::Item, _: Duration) -> Result<(), Self::Item> {
        self.try_push(value)
    }

    fn pop_timeout(&self, timeout: Duration) -> Option<Self::Item> {
        match self.semaphore.wait_timeout(timeout) {
            true => Some(self.queue.lock().pop_front().unwrap()),
            false => None,
        }
    }
}
#[async_trait]
impl<T, S, CS> AsyncTimeoutQueue for SemaphoreQueue<T, S, CS>
where
    T: Send,
    S: AsyncTimeoutSemaphore + Send + Sync,
    CS: ThreadFunctions,
{
    async fn push_timeout_async(&self, value: Self::Item, _: Duration) -> Result<(), Self::Item> {
        self.try_push(value)
    }

    async fn pop_timeout_async(&self, timeout: Duration) -> Option<Self::Item> {
        match self.semaphore.wait_timeout_async(timeout).await {
            true => Some(self.queue.lock().pop_front().unwrap()),
            false => None,
        }
    }
}
