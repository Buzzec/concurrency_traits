//! Generic mutex that only needs lock and unlock functionality to be auto
//! implemented.

#[cfg(feature = "alloc")]
mod mutex_alloc;

#[cfg(feature = "alloc")]
pub use mutex_alloc::*;

use crate::{EnsureSend, EnsureSync};

use core::cell::UnsafeCell;
use core::future::Future;
use core::ops::{Deref, DerefMut};
use core::time::Duration;

/// A non-blocking mutex with try functions.
///
/// ## Implementation
/// It is recommended to implement [`TryMutexSized`] if the implement-ee can be
/// sized.
pub trait TryMutex<'a> {
    /// The item stored in the mutex
    type Item: ?Sized;
    /// The guard for the mutex
    type Guard: DerefMut<Target = Self::Item>;

    /// Tries to lock the mutex, returning `None` if not possible.
    fn try_lock(&'a self) -> Option<Self::Guard>;
}
/// The functions for [`TryMutex`] that only work for sized types.
/// Separated to allow [`TryMutex`] to be a trait object.
pub trait TryMutexSized<'a>: Sized + TryMutex<'a> {
    /// Runs the function the value in the mutex if available immediately.
    ///
    /// ## Implementation
    /// Should be overwritten by implementors if can be more optimal than
    /// creating a guard
    fn try_lock_func<O>(&'a self, func: impl FnOnce(Option<&mut Self::Item>) -> O) -> O {
        match self.try_lock() {
            None => func(None),
            Some(mut guard) => func(Some(guard.deref_mut())),
        }
    }
}
/// A Generic Mutex trait
///
/// ## Implementation
/// It is recommended to implement [`MutexSized`] if the implement-ee can be
/// sized.
pub trait Mutex<'a>: TryMutex<'a> {
    /// Locks the mutex, blocking until successful
    fn lock(&'a self) -> Self::Guard;
}
/// The functions for [`Mutex`] that only work for sized types.
/// Separated to allow [`Mutex`] to be a trait object.
pub trait MutexSized<'a>: Mutex<'a> + TryMutexSized<'a> {
    /// Runs the function on the value in the mutex.
    ///
    /// ## Implementation
    /// Should be overwritten by implementors if can be more optimal than
    /// creating a guard
    fn lock_func<O>(&'a self, func: impl FnOnce(&mut Self::Item) -> O) -> O {
        func(self.lock().deref_mut())
    }
}
/// A generic async mutex trait
///
/// ## Implementation
/// It is recommended to implement [`AsyncMutexSized`] if the implement-ee can
/// be sized.
pub trait AsyncMutex<'a>: TryMutex<'a> {
    /// The guard for this async mutex
    type AsyncGuard: DerefMut<Target = Self::Item>;
    /// The future that the [`AsyncMutex::lock_async`] function returns
    type LockFuture: Future<Output = Self::AsyncGuard>;

    /// Locks the mutex asynchronously, returning a future with the guard.
    fn lock_async(&'a self) -> Self::LockFuture;
}

/// A mutex that can timeout for locking
///
/// ## Implementation
/// It is recommended to implement [`TimeoutMutexSized`] if the implement-ee can
/// be sized.
pub trait TimeoutMutex<'a>: TryMutex<'a> {
    /// Locks the mutex blocking for timeout or until locked
    fn lock_timeout(&'a self, timeout: Duration) -> Option<Self::Guard>;
}
/// The functions for [`TimeoutMutex`] that only work for sized types.
/// Separated to allow [`TimeoutMutex`] to be a trait object.
pub trait TimeoutMutexSized<'a>: Sized + TimeoutMutex<'a> + TryMutexSized<'a> {
    /// Attempts to lock the mutex before timeout has passed and runs func on
    /// the result
    ///
    /// ## Implementation
    /// Should be overwritten by implementors if can be more optimal than
    /// creating a guard
    fn lock_timeout_func<O>(
        &'a self,
        timeout: Duration,
        func: impl FnOnce(Option<&mut Self::Item>) -> O,
    ) -> O {
        match self.lock_timeout(timeout) {
            None => func(None),
            Some(mut guard) => func(Some(guard.deref_mut())),
        }
    }
}
/// An async mutex that locking can timeout on.
///
/// ## Implementation
/// It is recommended to implement [`AsyncTimeoutMutexSized`] if the
/// implement-ee can be sized.
pub trait AsyncTimeoutMutex<'a>: AsyncMutex<'a> {
    /// The future returned by `lock_timeout_async`
    type LockTimeoutFuture: Future<Output = Option<Self::AsyncGuard>>;
    /// Locks the mutex asynchronously with a timeout.
    fn lock_timeout_async(&'a self, timeout: Duration) -> Self::LockTimeoutFuture;
}

/// A raw mutex that can be tried and holds no data.
pub trait RawTryMutex {
    /// Locks the mutex, non-blocking. Returns true if locked.
    fn try_lock(&self) -> bool;
    /// # Safety
    /// Must only be called when sure that no references exist to contained
    /// data.
    unsafe fn unlock(&self);
}
/// A raw mutex that hold no data but the lock itself.
pub trait RawMutex: RawTryMutex {
    /// Locks the mutex, blocking.
    fn lock(&self);
}
/// A raw async mutex that hold no data but the lock itself.
pub trait RawAsyncMutex: RawTryMutex {
    /// The future returned by [`RawAsyncMutex::lock_async`]
    type LockFuture: Future<Output = ()>;
    /// Locks the mutex asynchronously
    fn lock_async(&self) -> Self::LockFuture;
}
/// A raw mutex that can be timed out and holds no data.
pub trait RawTimeoutMutex: RawMutex {
    /// Locks the mutex on a timeout. Returns true if locked.
    fn lock_timeout(&self, timeout: Duration) -> bool;
}
/// A raw async mutex that can be timed out and holds no data.
pub trait RawAsyncTimeoutMutex: RawAsyncMutex {
    /// The future returned by [`RawAsyncTimeoutMutex::lock_timeout_async`]
    type LockTimeoutFuture: Future<Output = bool>;
    /// Locks the mutex on a timeout asynchronously. Returns true if locked.
    fn lock_timeout_async(&self, timeout: Duration) -> Self::LockTimeoutFuture;
}

/// A Mutex based on a given [`RawTryMutex`]
#[derive(Debug)]
pub struct CustomMutex<T, M: ?Sized> {
    data: UnsafeCell<T>,
    raw_mutex: M,
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
impl<'a, T, M> TryMutex<'a> for CustomMutex<T, M>
where
    T: 'a,
    M: RawTryMutex + 'a,
{
    type Item = T;
    type Guard = CustomMutexGuard<'a, T, M>;

    fn try_lock(&'a self) -> Option<Self::Guard> {
        match self.raw_mutex.try_lock() {
            true => Some(CustomMutexGuard { mutex: self }),
            false => None,
        }
    }
}
impl<'a, T, M> TryMutexSized<'a> for CustomMutex<T, M>
where
    T: 'a,
    M: RawTryMutex + 'a,
{
    fn try_lock_func<O>(&'a self, func: impl FnOnce(Option<&mut Self::Item>) -> O) -> O {
        match self.raw_mutex.try_lock() {
            true => unsafe {
                let out = func(Some(&mut *self.data.get()));
                self.raw_mutex.unlock();
                out
            },
            false => func(None),
        }
    }
}
impl<'a, T, M> Mutex<'a> for CustomMutex<T, M>
where
    T: 'a,
    M: RawMutex + 'a,
{
    fn lock(&'a self) -> Self::Guard {
        self.raw_mutex.lock();
        CustomMutexGuard { mutex: self }
    }
}

impl<'a, T, M> MutexSized<'a> for CustomMutex<T, M>
where
    T: 'a,
    M: RawMutex + 'a,
{
    fn lock_func<O>(&'a self, func: impl FnOnce(&mut Self::Item) -> O) -> O {
        self.raw_mutex.lock();
        let out = func(unsafe { &mut *self.data.get() });
        unsafe { self.raw_mutex.unlock() }
        out
    }
}
impl<'a, T, M> TimeoutMutex<'a> for CustomMutex<T, M>
where
    T: 'a,
    M: RawTimeoutMutex + 'a,
{
    fn lock_timeout(&'a self, timeout: Duration) -> Option<Self::Guard> {
        match self.raw_mutex.lock_timeout(timeout) {
            true => Some(CustomMutexGuard { mutex: self }),
            false => None,
        }
    }
}
impl<'a, T, M> TimeoutMutexSized<'a> for CustomMutex<T, M>
where
    T: 'a,
    M: RawTimeoutMutex + 'a,
{
    fn lock_timeout_func<O>(
        &'a self,
        timeout: Duration,
        func: impl FnOnce(Option<&mut Self::Item>) -> O,
    ) -> O {
        match self.raw_mutex.lock_timeout(timeout) {
            true => unsafe {
                let out = func(Some(&mut *self.data.get()));
                self.raw_mutex.unlock();
                out
            },
            false => func(None),
        }
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
    mutex: &'a CustomMutex<T, M>,
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
    impl RawTryMutex for AtomicTryMutex {
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
    impl RawTryMutex for TestMutex {
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
    impl RawMutex for TestMutex {
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
