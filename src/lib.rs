//! Traits for concurrent primitives.

#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![warn(missing_docs, missing_debug_implementations, unused_import_braces)]
#![cfg_attr(feature = "nightly", feature(option_result_unwrap_unchecked))]

#[cfg(feature = "alloc")]
extern crate alloc;

mod mutex;
mod queue;
mod rw_lock;
mod stack;

#[cfg(feature = "alloc")]
mod alloc_impls;
mod impls;

pub use mutex::*;
pub use queue::*;
pub use rw_lock::*;
pub use stack::*;

use core::convert::Infallible;
use core::ops::{Add, Sub};
use core::time::Duration;

trait EnsureSend: Send {}
trait EnsureSync: Sync {}

/// Functions to interact with system time.
pub trait TimeFunctions {
    /// The type of an instant for this system. Analog for [`std::time::Instant`].
    type InstantType: Add<Duration, Output = Self::InstantType>
        + Sub<Duration, Output = Self::InstantType>
        + Sub<Self::InstantType, Output = Duration>
        + Ord
        + Copy;

    /// Get the current instant. Analog for [`std::time::Instant::now`].
    fn current_time(&self) -> Self::InstantType;
}
/// Functions to allow the current thread to interact in ways a thread might need to.
pub trait ThreadFunctions {
    /// Sleeps the current thread for a specified duration. Analog for [`std::thread::sleep`].
    fn sleep(&self, duration: Duration);
    /// Yields the current thread to the OS. Analog for [`std::thread::yield_now`].
    fn yield_now(&self);
}
/// Functions to spawn new threads. If infallibility is required look to [`ThreadSpawner`]. If a result is needed from the launched thread look to [`TryResultThreadSpawner`] or [`ResultThreadSpawner`]. `O` is the result of the thread function.
pub trait TryThreadSpawner<O>
where
    O: Send + 'static,
{
    /// The handle that is returned from spawning. Analog to [`std::thread::JoinHandle`].
    type ThreadHandle: ThreadHandle;
    /// The error that can occur from starting the thread.
    type SpawnError;

    /// Attempts to spawn a thread returning a result of [`Self::ThreadHandle`] and [`Self::SpawnError`].
    fn try_spawn(
        &self,
        func: impl FnOnce() -> O + 'static + Send,
    ) -> Result<Self::ThreadHandle, Self::SpawnError>;
}
/// Same as a [`TryThreadSpawner`] with an [`Infallible`] [`TryThreadSpawner::SpawnError`]. This is auto-implemented with [`TryThreadSpawner`] when possible. If a result is needed from the launched thread look to [`ResultThreadSpawner`].
pub trait ThreadSpawner<O>: Sized + TryThreadSpawner<O, SpawnError = Infallible>
where
    O: Send + 'static,
{
    /// Spawns a thread returning a [`Self::ThreadHandle`]. Analog to [`std::thread::spawn`]. Will be faster on nightly due to [`Result::unwrap_unchecked`].
    fn spawn(&self, func: impl FnOnce() -> O + 'static + Send) -> Self::ThreadHandle {
        #[cfg(not(feature = "nightly"))]
        {
            self.try_spawn(func).unwrap()
        }
        #[cfg(feature = "nightly")]
        unsafe {
            self.try_spawn(func).unwrap_unchecked()
        }
    }
}
impl<T, O> ThreadSpawner<O> for T
where
    T: TryThreadSpawner<O, SpawnError = Infallible>,
    O: Send + 'static,
{
}
/// Named version of [`TryThreadSpawner`] where the handle is a [`TryJoinableHandle`]. Auto implemented.
pub trait TryResultThreadSpawner<O>: TryThreadSpawner<O>
where
    Self::ThreadHandle: TryJoinableHandle<Output = O>,
    O: Send + 'static,
{
}
impl<T, O> TryResultThreadSpawner<O> for T
where
    T: TryThreadSpawner<O>,
    T::ThreadHandle: TryJoinableHandle<Output = O>,
    O: Send + 'static,
{
}
/// Named version of [`ThreadSpawner`] where the handle is a [`TryJoinableHandle`]. Auto implemented.
pub trait ResultThreadSpawner<O>: ThreadSpawner<O>
where
    Self::ThreadHandle: TryJoinableHandle<Output = O>,
    O: Send + 'static,
{
}
impl<T, O> ResultThreadSpawner<O> for T
where
    T: ThreadSpawner<O>,
    T::ThreadHandle: TryJoinableHandle<Output = O>,
    O: Send + 'static,
{
}
/// Functions to allow parking functionality for threads.
pub trait ThreadParker {
    /// The type of a thread portable id. Analog for [`std::thread::Thread`].
    type ThreadId;

    /// Parks the current thread. Analog for [`std::thread::park`].
    fn park(&self);
    /// Unparks a thread. Analog for [`std::thread::Thread::unpark`].
    fn unpark(&self, thread: Self::ThreadId);
}
/// Functions to allow parking functionality with timeout for threads.
pub trait ThreadTimeoutParker: ThreadParker {
    /// Parks the current thread with a timeout. Analog to [`std::thread::park_timeout`].
    fn park_timeout(&self, timeout: Duration);
}
/// A handle to a spawned thread. Analog for [`std::thread::JoinHandle`].
pub trait ThreadHandle {
    /// The type of a thread portable id. Analog for [`std::thread::Thread`].
    type ThreadId;

    /// Gets a thread id from this handle. Analog for [`std::thread::JoinHandle::thread`].
    fn thread_id(&self) -> &Self::ThreadId;
}
/// A handle to a spawned thread that can be joined, blocking the current thread until the target is finished. Analog for [`std::thread::JoinHandle`]. If infallibility is needed look to [`JoinableHandle`].
pub trait TryJoinableHandle: ThreadHandle {
    /// The output of joining this thread.
    type Output;
    /// The possible error when joining this thread,
    type ThreadError;

    /// Tries to join the target thread blocking the current thread. Analog for [`std::thread::JoinHandle::join`].
    fn try_join(self) -> Result<Self::Output, Self::ThreadError>;
}
/// A handle to a spawned thread that can be joined infallibly. Auto-implemented with [`TryJoinableHandle`] where possible.
pub trait JoinableHandle: Sized + TryJoinableHandle<ThreadError = Infallible> {
    /// Joins the target thread blocking the current thread.
    #[inline]
    fn join(self) -> Self::Output {
        #[cfg(not(feature = "nightly"))]
        {
            self.try_join().unwrap()
        }
        #[cfg(feature = "nightly")]
        unsafe {
            self.try_join().unwrap_unchecked()
        }
    }
}
impl<T> JoinableHandle for T where T: TryJoinableHandle<ThreadError = Infallible> {}

/// A full concurrent system with all functions accessible by reference. This Trait should be implemented where possible.
pub trait ConcurrentSystem<O>: 'static
where
    Self: TimeFunctions
        + ThreadFunctions
        + TryThreadSpawner<O>
        + ThreadParker<ThreadId = <Self::ThreadHandle as ThreadHandle>::ThreadId>,
    O: Send + 'static,
{
}

/// Std implementations for [`TimeFunctions`], [`ThreadFunctions], [`TryThreadSpawner`], and [`ThreadParker`].
#[cfg(feature = "std")]
#[derive(Copy, Clone, Debug)]
pub struct StdThreadFunctions;
#[cfg(feature = "std")]
mod std_thread_impls {
    use super::*;
    impl TimeFunctions for StdThreadFunctions {
        type InstantType = std::time::Instant;

        #[inline]
        fn current_time(&self) -> Self::InstantType {
            std::time::Instant::now()
        }
    }
    impl ThreadFunctions for StdThreadFunctions {
        #[inline]
        fn sleep(&self, duration: Duration) {
            std::thread::sleep(duration)
        }

        fn yield_now(&self) {
            std::thread::yield_now()
        }
    }
    impl<O> TryThreadSpawner<O> for StdThreadFunctions
    where
        O: Send + 'static,
    {
        type ThreadHandle = std::thread::JoinHandle<O>;
        type SpawnError = Infallible;

        fn try_spawn(
            &self,
            func: impl FnOnce() -> O + 'static + Send,
        ) -> Result<Self::ThreadHandle, Self::SpawnError> {
            Ok(std::thread::spawn(func))
        }
    }
    impl ThreadParker for StdThreadFunctions {
        type ThreadId = std::thread::Thread;

        #[inline]
        fn park(&self) {
            std::thread::park()
        }

        #[inline]
        fn unpark(&self, thread: Self::ThreadId) {
            thread.unpark()
        }
    }
    impl<O> ThreadHandle for std::thread::JoinHandle<O> {
        type ThreadId = std::thread::Thread;

        #[inline]
        fn thread_id(&self) -> &Self::ThreadId {
            self.thread()
        }
    }
    impl<O> TryJoinableHandle for std::thread::JoinHandle<O> {
        type Output = O;
        type ThreadError = Box<dyn std::any::Any + Send + 'static>;

        #[inline]
        fn try_join(self) -> Result<Self::Output, Self::ThreadError> {
            self.join()
        }
    }
    impl<O> ConcurrentSystem<O> for StdThreadFunctions where O: Send + 'static {}
}

// TODO: Replace future associated types and boxed futures with existential types when stabilized https://rust-lang.github.io/rfcs/2071-impl-trait-existential-types.html
