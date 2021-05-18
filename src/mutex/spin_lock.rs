use crate::mutex::{CustomMutex, RawMutex, RawTryMutex, RawAtomicMutex, RawTimeoutMutex};
use crate::{ThreadFunctions, TimeFunctions};
use core::marker::PhantomData;
use std::time::Duration;

/// A [`SpinLock`] that uses std functions.
#[cfg(feature = "std")]
pub type SpinLockStd<T> = SpinLock<T, crate::StdThreadFunctions>;

/// A lock that spins while being locked. Should only be locked for very short operations.
pub type SpinLock<T, CS> = CustomMutex<T, RawSpinLock<CS>>;
/// The raw portion of [`SpinLock`].
#[derive(Debug)]
pub struct RawSpinLock<CS> {
    lock: RawAtomicMutex,
    phantom_cs: PhantomData<fn() -> CS>,
}
impl<CS> Default for RawSpinLock<CS> {
    fn default() -> Self {
        Self {
            lock: RawAtomicMutex::default(),
            phantom_cs: Default::default(),
        }
    }
}
unsafe impl<CS> RawTryMutex for RawSpinLock<CS> {
    fn try_lock(&self) -> bool {
        self.lock.try_lock()
    }

    unsafe fn unlock(&self) {
        self.lock.unlock()
    }
}
unsafe impl<CS> RawMutex for RawSpinLock<CS>
where
    CS: ThreadFunctions,
{
    fn lock(&self) {
        while !self.lock.try_lock(){
            CS::yield_now()
        }
    }
}
unsafe impl<CS> RawTimeoutMutex for RawSpinLock<CS> where CS: ThreadFunctions + TimeFunctions{
    fn lock_timeout(&self, timeout: Duration) -> bool {
        let end = CS::current_time() + timeout;
        while end > CS::current_time(){
            if self.lock.try_lock(){
                return true;
            }
        }
        false
    }
}
