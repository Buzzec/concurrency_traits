//! Traits for concurrent primiatives.

#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![warn(missing_docs, missing_debug_implementations, unused_import_braces)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod mutex;
mod queue;
mod rw_lock;
mod stack;

mod impls;
#[cfg(feature = "alloc")]
mod alloc_impls;

pub use mutex::*;
pub use queue::*;
pub use rw_lock::*;
pub use stack::*;

trait EnsureSend: Send{}
trait EnsureSync: Sync{}

/// A spawner for new threads.
pub trait ThreadSpawner{
    /// The return value from [`ThreadSpawner::spawn`]
    type SpawnReturn;
    /// Spawns a new thread running `function`
    fn spawn(self, function: impl FnOnce() + 'static + Send) -> Self::SpawnReturn;
}

// TODO: Replace future associated types and boxed futures with existential types when stabilized https://rust-lang.github.io/rfcs/2071-impl-trait-existential-types.html
