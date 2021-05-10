use crate::mutex::{CustomMutex, RawMutex, RawTryMutex};
use crate::ThreadFunctions;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};

/// A [`SpinLock`] that uses std functions.
#[cfg(feature = "std")]
pub type SpinLockStd<T> = SpinLock<T, crate::StdThreadFunctions>;

/// A lock that spins while being locked. Should only be locked for very short operations.
pub type SpinLock<T, CS> = CustomMutex<T, RawSpinLock<CS>>;
/// The raw portion of [`SpinLock`].
#[derive(Debug)]
pub struct RawSpinLock<CS> {
    locked: AtomicBool,
    phantom_cs: PhantomData<fn() -> CS>,
}
impl<CS> Default for RawSpinLock<CS> {
    fn default() -> Self {
        Self {
            locked: AtomicBool::new(false),
            phantom_cs: Default::default(),
        }
    }
}
unsafe impl<CS> RawTryMutex for RawSpinLock<CS> {
    fn try_lock(&self) -> bool {
        !self.locked.swap(true, Ordering::AcqRel)
    }

    unsafe fn unlock(&self) {
        #[cfg(debug_assertions)]
        {
            assert!(self.locked.swap(false, Ordering::AcqRel));
        }
        #[cfg(not(debug_assertions))]
        {
            self.locked.store(false, Ordering::Release);
        }
    }
}
unsafe impl<CS> RawMutex for RawSpinLock<CS>
where
    CS: ThreadFunctions,
{
    fn lock(&self) {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_err()
        {
            CS::yield_now()
        }
    }
}
