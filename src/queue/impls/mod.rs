#[cfg(feature = "alloc")]
mod full_async_queue;
#[cfg(feature = "alloc")]
pub use full_async_queue::*;

#[cfg(feature = "alloc")]
mod park_queue;
#[cfg(feature = "alloc")]
pub use park_queue::*;

#[cfg(feature = "impl_crossbeam")]
mod queue_crossbeam;
