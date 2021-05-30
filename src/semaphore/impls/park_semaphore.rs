use crate::mutex::{Mutex, SpinLock};
use crate::semaphore::{ReadoutSemaphore, Semaphore, TimeoutSemaphore, TrySemaphore};
use crate::{ThreadFunctions, ThreadParker, ThreadTimeoutParker, TimeFunctions};
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::fmt::Debug;
use core::ops::{AddAssign, SubAssign};
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use num::{One, Zero};

/// A semaphore based on thread parking
#[derive(Debug)]
pub struct ParkSemaphore<C, CS>
where
    CS: ThreadParker,
{
    inner: SpinLock<ParkSemaphoreInner<C, CS>, CS>,
}
impl<C, CS> ParkSemaphore<C, CS>
where
    CS: ThreadParker,
{
    /// Creates a new [`ParkSemaphore`] from a `start_count`.
    pub fn new(start_count: C) -> Self {
        Self {
            inner: SpinLock::new(ParkSemaphoreInner {
                count: start_count,
                parkers: Default::default(),
            }),
        }
    }
}
impl<C, CS> Default for ParkSemaphore<C, CS>
where
    C: Zero,
    CS: ThreadParker,
{
    fn default() -> Self {
        Self::new(C::zero())
    }
}
unsafe impl<C, CS> TrySemaphore for ParkSemaphore<C, CS>
where
    C: Zero + One + AddAssign + SubAssign,
    CS: ThreadParker + ThreadFunctions,
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
            if let Some((thread_id, should_wake)) = guard.parkers.pop_front() {
                should_wake.store(true, Ordering::Release);
                CS::unpark(thread_id);
                return;
            }
        }
        guard.count += C::one();
    }
}
unsafe impl<C, CS> Semaphore for ParkSemaphore<C, CS>
where
    C: Zero + One + AddAssign + SubAssign,
    CS: ThreadParker + ThreadFunctions,
{
    fn wait(&self) {
        let mut guard = self.inner.lock();
        if !guard.count.is_zero() {
            guard.count -= C::one();
            return;
        }
        let should_wake = Arc::new(AtomicBool::new(false));
        guard
            .parkers
            .push_back((CS::current_thread(), should_wake.clone()));
        drop(guard);
        loop {
            if should_wake.load(Ordering::Acquire) {
                break;
            }
            CS::park();
        }
    }
}
unsafe impl<C, CS> TimeoutSemaphore for ParkSemaphore<C, CS>
where
    C: Zero + One + AddAssign + SubAssign,
    CS: ThreadTimeoutParker + TimeFunctions + ThreadFunctions,
{
    fn wait_timeout(&self, timeout: Duration) -> bool {
        let mut guard = self.inner.lock();
        if !guard.count.is_zero() {
            guard.count -= C::one();
            return true;
        }
        let should_wake = Arc::new(AtomicBool::new(false));
        guard
            .parkers
            .push_back((CS::current_thread(), should_wake.clone()));
        drop(guard);
        let end = CS::current_time() + timeout;
        loop {
            if CS::current_time() >= end {
                return false;
            }
            if should_wake.load(Ordering::Acquire) {
                return true;
            }
            CS::park_timeout(end - CS::current_time());
        }
    }
}
impl<C, CS> ReadoutSemaphore for ParkSemaphore<C, CS>
where
    C: Zero + One + AddAssign + SubAssign + Copy,
    CS: ThreadParker + ThreadFunctions,
{
    type Count = C;

    fn count(&self) -> Self::Count {
        self.inner.lock().count
    }
}

#[derive(Debug)]
struct ParkSemaphoreInner<C, CS>
where
    CS: ThreadParker,
{
    count: C,
    parkers: VecDeque<(CS::ThreadId, Arc<AtomicBool>)>,
}
