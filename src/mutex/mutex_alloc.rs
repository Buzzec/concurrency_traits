use crate::mutex::*;
use crate::queue::Queue;
use crate::TryThreadSpawner;
use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use async_trait::async_trait;
use simple_futures::complete_future::{CompleteFuture, CompleteFutureHandle};

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
    ) -> Result<(Self, TS::ThreadHandle), TS::SpawnError>
    where
        TS: TryThreadSpawner<()>,
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
            TS::try_spawn(move || Self::thread_function(raw_mutex_clone))?,
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
unsafe impl<M, Q> RawTryMutex for RawCustomAsyncMutex<M, Q>
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
unsafe impl<M, Q> RawMutex for RawCustomAsyncMutex<M, Q>
where
    M: 'static + RawMutex + Send + Sync,
    Q: 'static + Queue<Item = RawCustomAsyncMutexMessage> + Send + Sync,
{
    #[inline]
    fn lock(&self) {
        self.inner.raw_mutex.lock();
    }
}
#[async_trait]
unsafe impl<M, Q> RawAsyncMutex for RawCustomAsyncMutex<M, Q>
where
    M: 'static + RawMutex + Send + Sync,
    Q: 'static + Queue<Item = RawCustomAsyncMutexMessage> + Send + Sync,
{
    async fn lock_async(&self) {
        let future = CompleteFuture::new();
        self.inner
            .message_queue
            .try_push(RawCustomAsyncMutexMessage::Lock(future.get_handle()))
            .unwrap_or_else(|_| panic!("Could not add to message queue"));
        future.await;
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
