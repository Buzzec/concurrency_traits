//! Traits and implementations for semaphores.

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

mod readout;
pub use readout::*;

mod timeout;
pub use timeout::*;

mod r#try;
pub use r#try::*;

/// A generic semaphore.
///
/// # Safety
/// This trait is marked as unsafe to allow for safe code to rely on the
/// standard semaphore contract. [`Default`] implementations should initialize
/// the count at 0.
pub unsafe trait Semaphore: TrySemaphore {
    /// Decrements the count if able or blocks until able.
    fn wait(&self);
}
