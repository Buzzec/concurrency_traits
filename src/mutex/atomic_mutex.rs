use std::sync::atomic::{AtomicBool, Ordering};
use crate::mutex::{CustomMutex, RawTryMutex};
use crate::{EnsureSend, EnsureSync};

pub type AtomicMutex<T> = CustomMutex<T, RawAtomicMutex>;

pub struct RawAtomicMutex {
    locked: AtomicBool,
}
impl Default for RawAtomicMutex {
    fn default() -> Self {
        Self{ locked: AtomicBool::new(false) }
    }
}
unsafe impl RawTryMutex for RawAtomicMutex{
    #[inline]
    fn try_lock(&self) -> bool {
        !self.locked.swap(true, Ordering::AcqRel)
    }

    unsafe fn unlock(&self) {
        #[cfg(debug_assertions)] {
            assert!(self.locked.swap(false, Ordering::AcqRel));
        }
        #[cfg(not(debug_assertions))]{
            self.locked.store(false, Ordering::Release);
        }
    }
}
impl EnsureSend for RawAtomicMutex{}
impl EnsureSync for RawAtomicMutex{}
