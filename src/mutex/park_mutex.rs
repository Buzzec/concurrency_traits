use core::sync::atomic::{AtomicBool, Ordering};
use alloc::sync::Arc;
use atomic_swapping::option::AtomicSwapOption;
use core::time::Duration;
use alloc::collections::VecDeque;
use crate::{ThreadParker, ThreadFunctions, ThreadTimeoutParker, TimeFunctions};
use crate::mutex::{SpinLock, RawTryMutex, RawMutex, RawTimeoutMutex, Mutex, CustomMutex};

/// A mutex that relies on parking the thread that locks it.
pub type ParkMutex<T, CS> = CustomMutex<T, RawParkMutex<CS>>;

#[derive(Debug)]
struct RawParkMutexInner<CS> where CS: ThreadParker{
    /// Only needs to be set when unparking a thread.
    holder: Option<CS::ThreadId>,
    parkers: VecDeque<Arc<AtomicSwapOption<CS::ThreadId>>>,
}
/// The raw portion of [`ParkMutex`].
#[derive(Debug)]
pub struct RawParkMutex<CS> where CS: ThreadParker{
    locked: AtomicBool,
    inner: SpinLock<RawParkMutexInner<CS>, CS>,
}
impl<CS> Default for RawParkMutex<CS> where CS: ThreadParker{
    fn default() -> Self {
        Self{
            locked: AtomicBool::new(false),
            inner: SpinLock::new(RawParkMutexInner{
                holder: None,
                parkers: VecDeque::new(),
            })
        }
    }
}
unsafe impl<CS> RawTryMutex for RawParkMutex<CS> where CS: ThreadParker + ThreadFunctions, CS::ThreadId: Clone{
    fn try_lock(&self) -> bool {
        !self.locked.swap(true, Ordering::AcqRel)
    }

    unsafe fn unlock(&self) {
        let mut guard = self.inner.lock();
        loop {
            match guard.parkers.pop_front() {
                None => {
                    #[cfg(debug_assertions)] {
                        assert!(self.locked.swap(false, Ordering::AcqRel), "Lock was unlocked while not locked!");
                    }
                    #[cfg(not(debug_assertions))] {
                        self.locked.store(false, Ordering::Release);
                    }
                    guard.holder = None;
                    break;
                },
                Some(parker) => {
                    if let Some(parker) = parker.take(){
                        guard.holder = Some(parker.clone());
                        CS::unpark(parker);
                        break;
                    }
                }
            };
        }
    }
}
unsafe impl<CS> RawMutex for RawParkMutex<CS> where CS: ThreadParker + ThreadFunctions, CS::ThreadId: Eq + Clone{
    fn lock(&self) {
        let mut guard = self.inner.lock();
        if !self.try_lock() {
            let self_id = CS::current_thread();
            let self_id_swap = Arc::new(AtomicSwapOption::from_value(self_id.clone()));
            guard.parkers.push_back(self_id_swap);
            loop{
                drop(guard);
                CS::park();
                guard = self.inner.lock();
                if let Some(ref holder) = guard.holder{
                    if holder == &self_id{
                        // We have been unparked
                        break;
                    }
                }
            }
        }
    }
}
unsafe impl<CS> RawTimeoutMutex for RawParkMutex<CS> where CS: ThreadTimeoutParker + TimeFunctions + ThreadFunctions, CS::ThreadId: Clone + Eq{
    fn lock_timeout(&self, timeout: Duration) -> bool {
        let mut guard = self.inner.lock();
        if !self.try_lock(){
            true
        }
        else{
            let end = CS::current_time() + timeout;
            let self_id = CS::current_thread();
            let self_id_swap = Arc::new(AtomicSwapOption::from_value(self_id.clone()));
            guard.parkers.push_back(self_id_swap.clone());
            loop {
                drop(guard);
                CS::park_timeout(end - CS::current_time());
                guard = self.inner.lock();
                if let Some(ref holder) = guard.holder{
                    if holder == &self_id{
                        // We have been unparked
                        return true;
                    }
                }
                if CS::current_time() >= end{
                    #[cfg(debug_assertions)] {
                        assert!(self_id_swap.take().is_some());
                    }
                    #[cfg(not(debug_assertions))] {
                        self_id_swap.set(None);
                    }
                    return false;
                }
            }
        }
    }
}
