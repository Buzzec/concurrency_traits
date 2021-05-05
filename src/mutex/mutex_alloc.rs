use crate::mutex::*;
use crate::queue::Queue;
use crate::ThreadSpawner;
use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use core::future::Future;
use core::pin::Pin;
use core::time::Duration;
use simple_futures::complete_future::{CompleteFuture, CompleteFutureHandle};

/// The functions for [`AsyncMutex`] that only work for sized types.
/// Separated to allow [`AsyncMutex`] to be a trait object.
pub trait AsyncMutexSized<'a>: Sized + AsyncMutex<'a> + TryMutexSized<'a> {
    /// Locks the mutex and runs func on it asynchronously
    ///
    /// ## Implementation
    /// Should be overwritten by implementors if can be more optimal than
    /// creating a guard
    fn lock_async_func<F>(
        &'a self,
        func: impl FnOnce(&mut Self::Item) -> F + 'a,
    ) -> Pin<Box<dyn Future<Output = F::Output> + 'a>>
    where
        F: Future,
    {
        Box::pin(async move { func(self.lock_async().await.deref_mut()).await })
    }
}
/// The functions for [`AsyncTimeoutMutex`] that only work for sized types.
/// Separated to allow [`AsyncTimeoutMutex`] to be a trait object.
pub trait AsyncTimeoutMutexSized<'a>: Sized + AsyncTimeoutMutex<'a> {
    /// Locks the mutex with a timeout and runs a function on the result
    /// asynchronously
    ///
    /// ## Implementation
    /// Should be overwritten by implementors if can be more optimal than
    /// creating a guard
    fn lock_timeout_async_func<F>(
        &'a self,
        timeout: Duration,
        func: impl FnOnce(Option<&mut Self::Item>) -> F + 'a,
    ) -> Pin<Box<dyn Future<Output = F::Output> + 'a>>
    where
        F: Future + 'a,
    {
        Box::pin(async move {
            match self.lock_timeout_async(timeout).await {
                None => func(None).await,
                Some(mut guard) => func(Some(guard.deref_mut())).await,
            }
        })
    }
}

impl<'a, T, M> AsyncTimeoutMutex<'a> for CustomMutex<T, M>
where
    T: 'a,
    M: RawAsyncTimeoutMutex + 'a,
{
    type LockTimeoutFuture = Pin<Box<dyn Future<Output = Option<Self::AsyncGuard>> + 'a>>;

    fn lock_timeout_async(&'a self, timeout: Duration) -> Self::LockTimeoutFuture {
        Box::pin(async move {
            match self.raw_mutex.lock_timeout_async(timeout).await {
                true => Some(CustomMutexGuard { mutex: self }),
                false => None,
            }
        })
    }
}
impl<'a, T, M> AsyncTimeoutMutexSized<'a> for CustomMutex<T, M>
where
    T: 'a,
    M: RawAsyncTimeoutMutex + 'a,
{
    fn lock_timeout_async_func<F>(
        &'a self,
        timeout: Duration,
        func: impl FnOnce(Option<&mut Self::Item>) -> F + 'a,
    ) -> Pin<Box<dyn Future<Output = <F as Future>::Output> + 'a>>
    where
        F: Future,
    {
        Box::pin(async move {
            match self.raw_mutex.lock_timeout_async(timeout).await {
                true => unsafe {
                    let out = func(Some(&mut *self.data.get())).await;
                    self.raw_mutex.unlock();
                    out
                },
                false => func(None).await,
            }
        })
    }
}
impl<'a, T, M> AsyncMutex<'a> for CustomMutex<T, M>
where
    T: 'a,
    M: RawAsyncMutex + 'a,
{
    type AsyncGuard = CustomMutexGuard<'a, T, M>;
    type LockFuture = Pin<Box<dyn Future<Output = Self::AsyncGuard> + 'a>>;

    fn lock_async(&'a self) -> Self::LockFuture {
        Box::pin(async move {
            self.raw_mutex.lock_async().await;
            CustomMutexGuard { mutex: self }
        })
    }
}
impl<'a, T, M> AsyncMutexSized<'a> for CustomMutex<T, M>
where
    T: 'a,
    M: RawAsyncMutex + 'a,
{
    fn lock_async_func<F>(
        &'a self,
        func: impl FnOnce(&mut Self::Item) -> F + 'a,
    ) -> Pin<Box<dyn Future<Output = <F as Future>::Output> + 'a>>
    where
        F: Future,
    {
        Box::pin(async move {
            self.raw_mutex.lock_async().await;
            unsafe {
                let out = func(&mut *self.data.get()).await;
                self.raw_mutex.unlock();
                out
            }
        })
    }
}

/// A converter for turning a [`RawMutex`] into a [`RawAsyncMutex`]. Runs all
/// operations on own task.
#[derive(Debug)]
pub struct RawCustomAsyncMutex<M, Q> {
    inner: Arc<RawCustomAsyncMutexInner<M, Q>>,
}
impl<M, Q> RawCustomAsyncMutex<M, Q>
where
    M: 'static + RawMutex + Send + Sync,
    Q: 'static + Queue<Item = RawCustomAsyncMutexMessage> + Send + Sync,
{
    /// Creates a new [`RawCustomAsyncMutex`] from a [`RawMutex`] and a message
    /// queue.
    pub fn new<TS>(
        raw_mutex: M,
        message_queue: Q,
        spawner: TS,
    ) -> Result<(Self, TS::SpawnReturn), TS::SpawnError>
    where
        TS: ThreadSpawner,
    {
        let out = Self {
            inner: Arc::new(RawCustomAsyncMutexInner {
                raw_mutex,
                message_queue,
            }),
        };
        let raw_mutex_clone = Arc::downgrade(&out.inner);
        Ok((
            out,
            spawner.spawn(move || Self::thread_function(raw_mutex_clone))?,
        ))
    }

    fn thread_function(inner: Weak<RawCustomAsyncMutexInner<M, Q>>) {
        while let Some(inner) = inner.upgrade() {
            match inner.message_queue.pop() {
                RawCustomAsyncMutexMessage::Lock(future) => {
                    inner.raw_mutex.lock();
                    match future.complete() {
                        None => unsafe { inner.raw_mutex.unlock() },
                        Some(true) => panic!("Future was completed already!"),
                        Some(false) => {}
                    }
                } // RawCustomAsyncMutexMessage::LockTimeout { .. } => unreachable!(),
            }
        }
    }
}
impl<M, Q> RawTryMutex for RawCustomAsyncMutex<M, Q>
where
    M: 'static + RawMutex + Send + Sync,
    Q: 'static + Queue<Item = RawCustomAsyncMutexMessage> + Send + Sync,
{
    #[inline]
    fn try_lock(&self) -> bool {
        self.inner.raw_mutex.try_lock()
    }

    #[inline]
    unsafe fn unlock(&self) {
        self.inner.raw_mutex.unlock()
    }
}
impl<M, Q> RawMutex for RawCustomAsyncMutex<M, Q>
where
    M: 'static + RawMutex + Send + Sync,
    Q: 'static + Queue<Item = RawCustomAsyncMutexMessage> + Send + Sync,
{
    #[inline]
    fn lock(&self) {
        self.inner.raw_mutex.lock();
    }
}
impl<M, Q> RawAsyncMutex for RawCustomAsyncMutex<M, Q>
where
    M: 'static + RawMutex + Send + Sync,
    Q: 'static + Queue<Item = RawCustomAsyncMutexMessage> + Send + Sync,
{
    type LockFuture = CompleteFuture;

    fn lock_async(&self) -> Self::LockFuture {
        let future = CompleteFuture::new();
        self.inner
            .message_queue
            .try_push(RawCustomAsyncMutexMessage::Lock(future.get_handle()))
            .unwrap_or_else(|_| panic!("Could not add to message queue"));
        future
    }
}

#[derive(Debug)]
struct RawCustomAsyncMutexInner<M, Q> {
    raw_mutex: M,
    message_queue: Q,
}
/// The message used for [`RawCustomAsyncMutex`]
#[derive(Debug)]
pub enum RawCustomAsyncMutexMessage {
    /// A lock operation
    Lock(CompleteFutureHandle),
    // /// Not currently used but reserved for future designs
    // LockTimeout {
    //     /// The future the timeout is for
    //     future: ValueFutureHandle<bool>,
    //     /// When the timeout was called
    //     start: Instant,
    //     /// How long the timeout is for
    //     timeout: Duration,
    // },
}
