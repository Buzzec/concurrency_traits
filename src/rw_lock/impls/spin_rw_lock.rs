use crate::rw_lock::{
    CustomRwLock, RawAtomicRwLock, RawDowngradeRwLock, RawRwLock, RawTimeoutRwLock, RawTryRwLock,
    RawTryUpgradeRwLock, RawUpgradeRwLock, RawUpgradeTimeoutRwLock,
};
use crate::{ThreadFunctions, TimeFunctions};
use core::marker::PhantomData;
use core::time::Duration;

/// A read-write lock that spins to wait. Should only be locked for short
/// durations.
pub type SpinRwLock<T, CS> = CustomRwLock<T, RawSpinRwLock<CS>>;

/// The raw portion of [`SpinRwLock`].
#[derive(Debug)]
pub struct RawSpinRwLock<CS> {
    lock: RawAtomicRwLock,
    phantom_cs: PhantomData<fn() -> CS>,
}
impl<CS> Default for RawSpinRwLock<CS> {
    fn default() -> Self {
        Self {
            lock: Default::default(),
            phantom_cs: Default::default(),
        }
    }
}
unsafe impl<CS> RawTryRwLock for RawSpinRwLock<CS> {
    #[inline]
    fn try_add_reader(&self) -> bool {
        self.lock.try_add_reader()
    }

    #[inline]
    fn try_add_writer(&self) -> bool {
        self.lock.try_add_writer()
    }

    #[inline]
    unsafe fn remove_reader(&self) {
        self.lock.remove_reader()
    }

    #[inline]
    unsafe fn remove_writer(&self) {
        self.lock.remove_writer()
    }
}
unsafe impl<CS> RawRwLock for RawSpinRwLock<CS>
where
    CS: ThreadFunctions,
{
    fn add_reader(&self) {
        while !self.try_add_reader() {
            CS::yield_now();
        }
    }

    fn add_writer(&self) {
        while !self.try_add_writer() {
            CS::yield_now()
        }
    }
}
unsafe impl<CS> RawTryUpgradeRwLock for RawSpinRwLock<CS> {
    #[inline]
    unsafe fn try_upgrade(&self) -> bool {
        self.lock.try_upgrade()
    }
}
unsafe impl<CS> RawUpgradeRwLock for RawSpinRwLock<CS>
where
    CS: ThreadFunctions,
{
    unsafe fn upgrade(&self) {
        while !self.try_upgrade() {
            CS::yield_now();
        }
    }
}
unsafe impl<CS> RawUpgradeTimeoutRwLock for RawSpinRwLock<CS>
where
    CS: ThreadFunctions + TimeFunctions,
{
    unsafe fn upgrade_timeout(&self, timeout: Duration) -> bool {
        let end = CS::current_time() + timeout;
        while end > CS::current_time() {
            if self.try_upgrade() {
                return true;
            }
            CS::yield_now();
        }
        false
    }
}
unsafe impl<CS> RawDowngradeRwLock for RawSpinRwLock<CS> {
    #[inline]
    unsafe fn downgrade(&self) {
        self.lock.downgrade()
    }
}
unsafe impl<CS> RawTimeoutRwLock for RawSpinRwLock<CS>
where
    CS: ThreadFunctions + TimeFunctions,
{
    fn add_reader_timeout(&self, timeout: Duration) -> bool {
        let end = CS::current_time() + timeout;
        while end > CS::current_time() {
            if self.try_add_reader() {
                return true;
            }
            CS::yield_now();
        }
        false
    }

    fn add_writer_timeout(&self, timeout: Duration) -> bool {
        let end = CS::current_time() + timeout;
        while end > CS::current_time() {
            if self.try_add_writer() {
                return true;
            }
            CS::yield_now();
        }
        false
    }
}
