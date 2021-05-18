use std::sync::atomic::{AtomicUsize, Ordering};
use crate::rw_lock::{RawTryRwLock, CustomRwLock};

/// A read-write lock that only supports try operations ([`TryRwLock`](crate::rw_lock::TryRwLock)).
pub type AtomicRwLock<T> = CustomRwLock<T, RawAtomicRwLock>;

/// The raw portion of [`AtomicRwLock`].
#[derive(Debug)]
pub struct RawAtomicRwLock{
    /// 0 is writing,
    /// 1 is open,
    /// x is x - 1 readers,
    pub(in crate::rw_lock) read_count: AtomicUsize,
}
impl Default for RawAtomicRwLock{
    fn default() -> Self {
        Self{
            read_count: AtomicUsize::new(1),
        }
    }
}
unsafe impl RawTryRwLock for RawAtomicRwLock{
    fn try_add_reader(&self) -> bool {
        let mut count = self.read_count.load(Ordering::Acquire);
        loop{
            if count < 1{
                return false;
            }
            match self.read_count.compare_exchange_weak(count, count + 1, Ordering::AcqRel, Ordering::Acquire){
                Ok(_) => return true,
                Err(new_count) => count = new_count,
            }
        }
    }

    fn try_add_writer(&self) -> bool {
        let mut count = self.read_count.load(Ordering::Acquire);
        loop{
            if count != 1{
                return false;
            }
            match self.read_count.compare_exchange_weak(count, 0, Ordering::AcqRel, Ordering::Acquire){
                Ok(_) => return true,
                Err(new_count) => count = new_count,
            }
        }
    }

    unsafe fn remove_reader(&self) {
        #[cfg(debug_assertions)] {
            assert!(self.read_count.fetch_sub(1, Ordering::AcqRel) > 1);
        }
        #[cfg(not(debug_assertions))]{
            self.read_count.fetch_sub(1, Ordering::Release);
        }
    }

    unsafe fn remove_writer(&self) {
        #[cfg(debug_assertions)] {
            assert_eq!(self.read_count.swap(1, Ordering::AcqRel), 0);
        }
        #[cfg(not(debug_assertions))]{
            self.read_count.store(1, Ordering::Release);
        }
    }
}
