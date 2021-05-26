use crate::mutex::{CustomMutex, RawAsyncMutex, RawTryMutex};
use crate::queue::TryQueue;
use alloc::boxed::Box;
use async_trait::async_trait;
use core::sync::atomic::{AtomicBool, Ordering};
use simple_futures::complete_future::{CompleteFuture, CompleteFutureHandle};

/// A mutex that can only be accessed through async await or try operations.
/// ```
/// # #[cfg(feature = "std")]
/// # {
/// use std::ops::Deref;
/// use concurrency_traits::mutex::{FullAsyncMutex, AsyncMutex};
/// use std::task::{Context, Waker, Wake};
/// use std::sync::Arc;
/// use std::future::Future;
/// use concurrency_traits::queue::ParkQueueStd;
///
/// struct NullWaker;
/// impl Wake for NullWaker{
///     fn wake(self:Arc<Self>) {
///         println!("Wake!");
///     }
/// }
///
/// let mut future = Box::pin(async move {
///     let mutex = FullAsyncMutex::<_, ParkQueueStd<_>>::new(100usize);
///     let guard = mutex.lock_async().await;
///     assert_eq!(*guard.deref(), 100usize);
/// });
///
/// assert!(!future.as_mut().poll(&mut Context::from_waker(&Arc::new(NullWaker).into())).is_pending())
/// # }
/// ```
pub type FullAsyncMutex<T, Q> = CustomMutex<T, RawFullAsyncMutex<Q>>;

/// The raw portion of [`FullAsyncMutex`].
#[derive(Debug)]
pub struct RawFullAsyncMutex<Q> {
    locked: AtomicBool,
    waiting_queue: Q,
}
impl<Q> Default for RawFullAsyncMutex<Q>
where
    Q: Default,
{
    #[inline]
    fn default() -> Self {
        Self::from(Q::default())
    }
}
impl<Q> From<Q> for RawFullAsyncMutex<Q> {
    #[inline]
    fn from(from: Q) -> Self {
        Self {
            locked: AtomicBool::new(false),
            waiting_queue: from,
        }
    }
}
unsafe impl<Q> RawTryMutex for RawFullAsyncMutex<Q>
where
    Q: TryQueue<Item = CompleteFutureHandle>,
{
    #[inline]
    fn try_lock(&self) -> bool {
        !self.locked.swap(true, Ordering::AcqRel)
    }

    unsafe fn unlock(&self) {
        loop {
            if let Some(handle) = self.waiting_queue.try_pop() {
                let result = handle.complete();
                if result.is_some() {
                    debug_assert!(!result.unwrap());
                    return;
                }
            } else {
                #[cfg(debug_assertions)]
                {
                    assert!(self.locked.swap(false, Ordering::AcqRel));
                }
                #[cfg(not(debug_assertions))]
                {
                    self.locked.store(false, Ordering::Release);
                }
                return;
            }
        }
    }
}
#[async_trait]
unsafe impl<Q> RawAsyncMutex for RawFullAsyncMutex<Q>
where
    Q: TryQueue<Item = CompleteFutureHandle> + Sync,
{
    async fn lock_async(&self) {
        let future = CompleteFuture::new();
        match self.try_lock() {
            true => {
                let result = future.complete();
                debug_assert!(!result);
            }
            false => {
                self.waiting_queue
                    .try_push(future.get_handle())
                    .expect("Could not push handle!");
                // if was unlocked completely between try_lock and try_push
                if self.try_lock() {
                    unsafe { self.unlock() }
                }
            }
        }
        future.await
    }
}
