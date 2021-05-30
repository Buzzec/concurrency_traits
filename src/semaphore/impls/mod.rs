#[cfg(feature = "alloc")]
mod async_semaphore;
#[cfg(feature = "alloc")]
pub use async_semaphore::*;

mod atomic_semaphore;
pub use atomic_semaphore::*;

#[cfg(feature = "alloc")]
mod park_semaphore;
#[cfg(feature = "alloc")]
pub use park_semaphore::*;
