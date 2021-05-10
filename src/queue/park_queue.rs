use crate::mutex::{Mutex, SpinLock};
use crate::queue::{Queue, TimeoutQueue, TryQueue};
use crate::{ThreadFunctions, ThreadParker, ThreadTimeoutParker, TimeFunctions};
use alloc::collections::VecDeque;
use alloc::sync::{Arc, Weak};
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;

/// A [`ParkQueue`] that uses std functions.
#[cfg(feature = "std")]
pub type ParkQueueStd<T> = ParkQueue<T, crate::StdThreadFunctions>;

/// A queue based on [`VecDeque`]s and parking.
#[derive(Debug)]
pub struct ParkQueue<T, CS>
where
    CS: ThreadParker,
{
    inner: SpinLock<ParkQueueInner<T, CS>, CS>,
}
impl<T, CS> Default for ParkQueue<T, CS>
where
    CS: ThreadParker,
{
    fn default() -> Self {
        Self {
            inner: SpinLock::new(ParkQueueInner {
                queue: Default::default(),
                parkers: VecDeque::new(),
            }),
        }
    }
}
impl<T, CS> TryQueue for ParkQueue<T, CS>
where
    CS: ThreadParker + ThreadFunctions,
    CS::ThreadId: Clone,
{
    type Item = T;

    fn try_push(&self, value: Self::Item) -> Result<(), Self::Item> {
        let mut guard = self.inner.lock();
        guard.queue.push_back(value);
        while let Some(parker) = guard.parkers.pop_front() {
            if let Some(parker) = parker.upgrade() {
                parker.1.store(true, Ordering::Release);
                CS::unpark(parker.0.clone());
                break;
            }
        }
        Ok(())
    }

    fn try_pop(&self) -> Option<Self::Item> {
        self.inner.lock().queue.pop_front()
    }

    fn clear(&self) {
        self.inner.lock().queue.clear();
    }
}
impl<T, CS> Queue for ParkQueue<T, CS>
where
    CS: ThreadParker + ThreadFunctions,
    CS::ThreadId: Clone,
{
    fn push(&self, value: Self::Item) {
        self.try_push(value)
            .unwrap_or_else(|_| panic!("Try push should not fail!"));
    }

    fn pop(&self) -> Self::Item {
        let mut guard = self.inner.lock();
        if let Some(value) = guard.queue.pop_front() {
            return value;
        }
        let self_swap = Arc::new((CS::current_thread(), AtomicBool::new(false)));
        guard.parkers.push_back(Arc::downgrade(&self_swap));
        loop {
            drop(guard);
            CS::park();
            guard = self.inner.lock();
            if self_swap.1.load(Ordering::Acquire) {
                if let Some(value) = guard.queue.pop_front() {
                    return value;
                } else {
                    guard.parkers.push_front(Arc::downgrade(&self_swap));
                }
            }
        }
    }
}
impl<T, CS> TimeoutQueue for ParkQueue<T, CS>
where
    CS: ThreadTimeoutParker + ThreadFunctions + TimeFunctions,
    CS::ThreadId: Clone,
{
    fn push_timeout(&self, value: Self::Item, _timeout: Duration) -> Result<(), Self::Item> {
        self.try_push(value)
            .unwrap_or_else(|_| panic!("Try push should not fail!"));
        Ok(())
    }

    fn pop_timeout(&self, timeout: Duration) -> Option<Self::Item> {
        let end = CS::current_time() + timeout;
        let mut guard = self.inner.lock();
        if let Some(value) = guard.queue.pop_front() {
            return Some(value);
        }
        let self_swap = Arc::new((CS::current_thread(), AtomicBool::new(false)));
        guard.parkers.push_back(Arc::downgrade(&self_swap));
        loop {
            drop(guard);
            let current_time = CS::current_time();
            if current_time < end {
                CS::park_timeout(end - current_time);
            }
            guard = self.inner.lock();
            if self_swap.1.load(Ordering::Acquire) {
                if let Some(value) = guard.queue.pop_front() {
                    return Some(value);
                } else if CS::current_time() >= end {
                    return None;
                } else {
                    guard.parkers.push_front(Arc::downgrade(&self_swap));
                }
            }
            if CS::current_time() >= end {
                return None;
            }
        }
    }
}

#[derive(Debug)]
struct ParkQueueInner<T, CS>
where
    CS: ThreadParker,
{
    queue: VecDeque<T>,
    /// True if should wake
    parkers: VecDeque<Weak<(CS::ThreadId, AtomicBool)>>,
}

#[cfg(test)]
mod test {
    use crate::queue::test::{queue_test, try_queue_test};
    use crate::queue::ParkQueue;
    #[cfg(feature = "std")]
    use crate::StdThreadFunctions;

    #[cfg(feature = "std")]
    #[test]
    fn function_test() {
        try_queue_test(ParkQueue::<_, StdThreadFunctions>::default());
        queue_test(ParkQueue::<_, StdThreadFunctions>::default());
    }
}
