use crate::semaphore::{ReadoutSemaphore, TrySemaphore};
use core::sync::atomic::{AtomicUsize, Ordering};

/// A semaphore based on atomic operations.
#[derive(Debug)]
pub struct AtomicSemaphore {
    count: AtomicUsize,
}
impl AtomicSemaphore {
    /// Creates a new [`AtomicSemaphore`] from a `start_count`.
    pub fn new(start_count: usize) -> Self {
        Self {
            count: AtomicUsize::new(start_count),
        }
    }
}
impl Default for AtomicSemaphore {
    fn default() -> Self {
        Self::new(0)
    }
}
unsafe impl TrySemaphore for AtomicSemaphore {
    fn try_wait(&self) -> bool {
        let mut count = self.count.load(Ordering::Acquire);
        loop {
            if count == 0 {
                return false;
            }
            match self.count.compare_exchange_weak(
                count,
                count - 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return true,
                Err(new_count) => count = new_count,
            }
        }
    }

    fn signal(&self) {
        self.count.fetch_add(1, Ordering::AcqRel);
    }
}
impl ReadoutSemaphore for AtomicSemaphore {
    type Count = usize;

    fn count(&self) -> Self::Count {
        self.count.load(Ordering::Acquire)
    }
}
