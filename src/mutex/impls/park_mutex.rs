use crate::mutex::{CustomMutex, Mutex, RawMutex, RawTimeoutMutex, RawTryMutex, SpinLock};
use crate::{ThreadFunctions, ThreadParker, ThreadTimeoutParker, TimeFunctions};
use alloc::collections::VecDeque;
use alloc::sync::{Arc, Weak};
use core::ops::Deref;
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;

/// A [`ParkMutex`] that uses std functions.
#[cfg(feature = "std")]
pub type ParkMutexStd<T> = ParkMutex<T, crate::StdThreadFunctions>;

/// A [`RawParkMutex`] that uses std functions.
#[cfg(feature = "std")]
pub type RawParkMutexStd = RawParkMutex<crate::StdThreadFunctions>;

/// A mutex that relies on parking the thread that locks it. Uses [`RawParkMutex`]
pub type ParkMutex<T, CS> = CustomMutex<T, RawParkMutex<CS>>;

/// The raw portion of [`ParkMutex`].
#[derive(Debug)]
pub struct RawParkMutex<CS>
where
    CS: ThreadParker,
{
    locked: AtomicBool,
    inner: SpinLock<RawParkMutexInner<CS>, CS>,
}
impl<CS> Default for RawParkMutex<CS>
where
    CS: ThreadParker,
{
    fn default() -> Self {
        Self {
            locked: AtomicBool::new(false),
            inner: SpinLock::new(RawParkMutexInner {
                holder: None,
                parkers: VecDeque::new(),
            }),
        }
    }
}
unsafe impl<CS> RawTryMutex for RawParkMutex<CS>
where
    CS: ThreadParker + ThreadFunctions,
    CS::ThreadId: Clone,
{
    fn try_lock(&self) -> bool {
        !self.locked.swap(true, Ordering::AcqRel)
    }

    unsafe fn unlock(&self) {
        let mut guard = self.inner.lock();
        loop {
            match guard.parkers.pop_front() {
                None => {
                    #[cfg(debug_assertions)]
                    {
                        assert!(
                            self.locked.swap(false, Ordering::AcqRel),
                            "Lock was unlocked while not locked!"
                        );
                    }
                    #[cfg(not(debug_assertions))]
                    {
                        self.locked.store(false, Ordering::Release);
                    }
                    guard.holder = None;
                    break;
                }
                Some(parker) => {
                    if let Some(parker) = parker.upgrade() {
                        guard.holder = Some(parker.deref().clone());
                        CS::unpark(parker.deref().clone());
                        break;
                    }
                }
            };
        }
    }
}
unsafe impl<CS> RawMutex for RawParkMutex<CS>
where
    CS: ThreadParker + ThreadFunctions,
    CS::ThreadId: Eq + Clone,
{
    fn lock(&self) {
        let mut guard = self.inner.lock();
        if !self.try_lock() {
            let self_id = Arc::new(CS::current_thread());
            guard.parkers.push_back(Arc::downgrade(&self_id));
            loop {
                drop(guard);
                CS::park();
                guard = self.inner.lock();
                if let Some(ref holder) = guard.holder {
                    if holder == self_id.deref() {
                        // We have been unparked
                        break;
                    }
                }
            }
        }
    }
}
unsafe impl<CS> RawTimeoutMutex for RawParkMutex<CS>
where
    CS: ThreadTimeoutParker + TimeFunctions + ThreadFunctions,
    CS::ThreadId: Clone + Eq,
{
    fn lock_timeout(&self, timeout: Duration) -> bool {
        let mut guard = self.inner.lock();
        if !self.try_lock() {
            true
        } else {
            let end = CS::current_time() + timeout;
            let self_id = Arc::new(CS::current_thread());
            guard.parkers.push_back(Arc::downgrade(&self_id));
            loop {
                drop(guard);
                let current_time = CS::current_time();
                if end < current_time {
                    CS::park_timeout(end - current_time);
                }
                guard = self.inner.lock();
                if let Some(ref holder) = guard.holder {
                    if holder == self_id.deref() {
                        // We have been unparked
                        return true;
                    }
                }
                if CS::current_time() >= end {
                    return false;
                }
            }
        }
    }
}

#[derive(Debug)]
struct RawParkMutexInner<CS>
where
    CS: ThreadParker,
{
    /// Only needs to be set when unparking a thread.
    holder: Option<CS::ThreadId>,
    parkers: VecDeque<Weak<CS::ThreadId>>,
}
