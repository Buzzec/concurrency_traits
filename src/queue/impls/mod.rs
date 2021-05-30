#[cfg(feature = "alloc")]
mod semaphore_queue;
#[cfg(feature = "alloc")]
pub use semaphore_queue::*;

#[cfg(feature = "alloc")]
mod park_queue;
#[cfg(feature = "alloc")]
pub use park_queue::*;

#[cfg(feature = "impl_crossbeam")]
mod queue_crossbeam;
