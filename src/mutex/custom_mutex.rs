use core::cell::UnsafeCell;
use crate::{EnsureSend, EnsureSync};
use core::ops::{Deref, DerefMut};
use crate::mutex::RawTryMutex;

/// A Mutex based on a given [`RawTryMutex`]
#[derive(Debug)]
pub struct CustomMutex<T, M: ?Sized> {
    pub(super) data: UnsafeCell<T>,
    pub(super) raw_mutex: M,
}
impl<T, M> CustomMutex<T, M> {
    /// Creates a new `CustomMutex` with a `RawMutex`
    pub const fn from_raw(raw_mutex: M, data: T) -> Self {
        Self {
            raw_mutex,
            data: UnsafeCell::new(data),
        }
    }

    /// Creates a new mutex with a default raw
    pub fn new(data: T) -> Self
    where
        M: Default,
    {
        Self::from_raw(M::default(), data)
    }
}
impl<T, M> Default for CustomMutex<T, M>
where
    T: Default,
    M: Default,
{
    #[inline]
    fn default() -> Self {
        Self::new(T::default())
    }
}
impl<T, M> From<T> for CustomMutex<T, M>
where
    M: Default,
{
    fn from(from: T) -> Self {
        Self::new(from)
    }
}
impl<T, M> EnsureSend for CustomMutex<T, M>
where
    T: Send,
    M: Send,
{
}
unsafe impl<T, M> Sync for CustomMutex<T, M>
where
    T: Send,
    M: Sync,
{
}

/// A guard for a `CustomMutex`
#[derive(Debug)]
pub struct CustomMutexGuard<'a, T, M>
where
    M: RawTryMutex,
{
    pub(super) mutex: &'a CustomMutex<T, M>,
}
impl<'a, T, M> Deref for CustomMutexGuard<'a, T, M>
where
    M: RawTryMutex,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}
impl<'a, T, M> DerefMut for CustomMutexGuard<'a, T, M>
where
    M: RawTryMutex,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}
impl<'a, T, M> Drop for CustomMutexGuard<'a, T, M>
where
    M: RawTryMutex,
{
    fn drop(&mut self) {
        unsafe { self.mutex.raw_mutex.unlock() }
    }
}
impl<'a, T, M> EnsureSend for CustomMutexGuard<'a, T, M>
where
    T: Send,
    M: RawTryMutex + Send + Sync,
{
}
impl<'a, T, M> EnsureSync for CustomMutexGuard<'a, T, M>
where
    T: Send,
    M: RawTryMutex + Send + Sync,
{
}

#[cfg(test)]
mod test {
    use crate::mutex::{
        CustomMutex, Mutex, MutexSized, RawMutex, RawTryMutex, TryMutex, TryMutexSized,
    };
    use std::mem::swap;
    use std::ops::{Deref, DerefMut};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{self, Arc, Condvar};
    use std::thread::{spawn, yield_now};

    // True if unlocked
    struct AtomicTryMutex {
        unlocked: AtomicBool,
    }
    impl Default for AtomicTryMutex {
        fn default() -> Self {
            Self {
                unlocked: AtomicBool::new(true),
            }
        }
    }
    unsafe impl RawTryMutex for AtomicTryMutex {
        fn try_lock(&self) -> bool {
            self.unlocked.swap(false, Ordering::SeqCst)
        }

        unsafe fn unlock(&self) {
            assert!(!self.unlocked.swap(true, Ordering::SeqCst));
        }
    }
    #[test]
    fn try_mutex_test() {
        let custom_mutex: CustomMutex<_, AtomicTryMutex> = CustomMutex::new(100);
        assert_eq!(
            custom_mutex.try_lock().map(|guard| *guard.deref()),
            Some(100)
        );
        let mut guard = custom_mutex.try_lock().expect("Could not lock!");
        *guard.deref_mut() = 200;
        drop(guard);
        custom_mutex.try_lock_func(|guard| {
            let value = guard.expect("Could not lock");
            assert_eq!(*value, 200);
            *value = 300;
        });
        let guard = custom_mutex.try_lock().expect("Could not lock!");
        assert!(custom_mutex.try_lock().is_none());
        assert_eq!(*guard, 300);
    }

    struct TestMutex {
        unlocked: sync::Mutex<bool>,
        parkers: Condvar,
    }
    impl Default for TestMutex {
        fn default() -> Self {
            Self {
                unlocked: sync::Mutex::new(true),
                parkers: Condvar::new(),
            }
        }
    }
    unsafe impl RawTryMutex for TestMutex {
        fn try_lock(&self) -> bool {
            let mut out = false;
            swap(
                self.unlocked.lock().expect("Poisoned").deref_mut(),
                &mut out,
            );
            out
        }

        unsafe fn unlock(&self) {
            let mut out = true;
            swap(
                self.unlocked.lock().expect("Poisoned").deref_mut(),
                &mut out,
            );
            assert!(!out);
            self.parkers.notify_one();
        }
    }
    unsafe impl RawMutex for TestMutex {
        fn lock(&self) {
            let mut guard = self.unlocked.lock().expect("Poisoned");
            let mut out = false;
            swap(guard.deref_mut(), &mut out);
            if !out {
                drop(
                    self.parkers
                        .wait_while(guard, |val| {
                            let mut out = false;
                            swap(val, &mut out);
                            !out
                        })
                        .expect("Poisoned"),
                );
            }
        }
    }
    #[test]
    fn mutex_test() {
        let custom_mutex: CustomMutex<_, TestMutex> = CustomMutex::new(100);
        assert_eq!(
            custom_mutex.try_lock().map(|guard| *guard.deref()),
            Some(100)
        );
        let mut guard = custom_mutex.try_lock().expect("Could not lock!");
        *guard.deref_mut() = 200;
        drop(guard);
        custom_mutex.try_lock_func(|guard| {
            let value = guard.expect("Could not lock");
            assert_eq!(*value, 200);
            *value = 300;
        });
        let guard = custom_mutex.try_lock().expect("Could not lock!");
        assert!(custom_mutex.try_lock().is_none());
        assert_eq!(*guard, 300);
        drop(guard);
        assert_eq!(*custom_mutex.lock(), 300);
        let arc = Arc::new((custom_mutex, AtomicUsize::new(0)));
        let arc_clone = arc.clone();
        let guard = arc.0.lock();
        let handle = spawn(move || {
            arc_clone.0.lock_func(|val| {
                assert_eq!(*val, 300);
                *val = 400;
            });
            arc_clone.1.fetch_add(1, Ordering::SeqCst);
        });
        yield_now();
        assert_eq!(arc.1.load(Ordering::SeqCst), 0);
        drop(guard);
        handle.join().expect("Could not join");
        assert_eq!(arc.1.load(Ordering::SeqCst), 1);
        assert_eq!(*arc.0.lock(), 400);
    }
}
