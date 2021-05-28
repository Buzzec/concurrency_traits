//! Generic Mutex traits and implementations.

mod impls;
pub use impls::*;

#[cfg(feature = "alloc")]
mod r#async;
#[cfg(feature = "alloc")]
pub use r#async::*;

#[cfg(feature = "alloc")]
mod async_timeout;
#[cfg(feature = "alloc")]
pub use async_timeout::*;

mod custom;
pub use custom::*;

#[cfg(feature = "alloc")]
mod custom_async;
#[cfg(feature = "alloc")]
pub use custom_async::*;

mod timeout;
pub use timeout::*;

mod r#try;
pub use r#try::*;

use core::ops::DerefMut;

/// A raw mutex that hold no data but the lock itself.
pub unsafe trait RawMutex: RawTryMutex {
    /// Locks the mutex, blocking.
    fn lock(&self);
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
