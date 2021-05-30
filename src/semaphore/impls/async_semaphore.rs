use crate::mutex::{Mutex, SpinLock};
use crate::semaphore::{AsyncSemaphore, ReadoutSemaphore, TrySemaphore};
use crate::ThreadFunctions;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use async_trait::async_trait;
use core::fmt::Debug;
use core::ops::{AddAssign, SubAssign};
use num::{One, Zero};
use simple_futures::complete_future::{CompleteFuture, CompleteFutureHandle};

/// A semaphore that has asynchronous operations.
#[derive(Debug)]
pub struct FullAsyncSemaphore<C, CS> {
    inner: SpinLock<AsyncSemaphoreInner<C>, CS>,
}
impl<C, CS> FullAsyncSemaphore<C, CS> {
    /// Creates a new [`FullAsyncSemaphore`] from a starting count.
    pub fn new(start_count: C) -> Self {
        Self {
            inner: SpinLock::new(AsyncSemaphoreInner {
                count: start_count,
                waker_queue: Default::default(),
            }),
        }
    }
}
impl<C, CS> Default for FullAsyncSemaphore<C, CS>
where
    C: Zero,
{
    fn default() -> Self {
        Self::new(C::zero())
    }
}
unsafe impl<C, CS> TrySemaphore for FullAsyncSemaphore<C, CS>
where
    C: Zero + One + AddAssign + SubAssign,
    CS: ThreadFunctions,
{
    fn try_wait(&self) -> bool {
        let mut guard = self.inner.lock();
        if guard.count.is_zero() {
            false
        } else {
            guard.count -= C::one();
            true
        }
    }

    fn signal(&self) {
        let mut guard = self.inner.lock();
        if guard.count.is_zero() {
            while let Some(handle) = guard.waker_queue.pop_front() {
                if let Some(result) = handle.complete() {
                    assert!(result);
                    return;
                }
            }
        }
        guard.count += C::one();
    }
}
#[async_trait]
unsafe impl<C, CS> AsyncSemaphore for FullAsyncSemaphore<C, CS>
where
    C: Zero + One + AddAssign + SubAssign + Send,
    CS: ThreadFunctions,
{
    async fn wait_async(&self) {
        let mut guard = self.inner.lock();
        if !guard.count.is_zero() {
            guard.count -= C::one();
            return;
        }
        let future = CompleteFuture::new();
        guard.waker_queue.push_back(future.get_handle());
        drop(guard);
        future.await;
    }
}
impl<C, CS> ReadoutSemaphore for FullAsyncSemaphore<C, CS>
where
    C: Zero + One + AddAssign + SubAssign + Copy,
    CS: ThreadFunctions,
{
    type Count = C;

    fn count(&self) -> Self::Count {
        self.inner.lock().count
    }
}

#[derive(Debug)]
struct AsyncSemaphoreInner<C> {
    count: C,
    waker_queue: VecDeque<CompleteFutureHandle>,
}
