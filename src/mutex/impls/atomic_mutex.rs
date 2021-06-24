use crate::mutex::{CustomMutex, RawTryMutex};
use crate::{EnsureSend, EnsureSync};
use core::sync::atomic::{AtomicBool, Ordering};

/// A mutex based on an [`AtomicBool`]. Only supports try operations
/// ([`TryMutex`](crate::mutex::TryMutex)).
pub type AtomicMutex<T> = CustomMutex<T, RawAtomicMutex>;

/// The raw portion of [`AtomicMutex`].
#[derive(Debug)]
pub struct RawAtomicMutex {
    locked: AtomicBool,
}
impl Default for RawAtomicMutex {
    fn default() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }
}
unsafe impl RawTryMutex for RawAtomicMutex {
    #[inline]
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
impl EnsureSend for RawAtomicMutex {}
impl EnsureSync for RawAtomicMutex {}
