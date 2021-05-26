#[cfg(feature = "alloc")]
mod async_mutex;

#[cfg(feature = "alloc")]
pub use async_mutex::*;

mod atomic_mutex;
pub use atomic_mutex::*;

#[cfg(feature = "impl_parking_lot")]
mod mutex_parking_lot;

#[cfg(feature = "alloc")]
mod park_mutex;

#[cfg(feature = "alloc")]
pub use park_mutex::*;

mod spin_lock;
pub use spin_lock::*;

#[cfg(feature = "std")]
mod std_mutex;
