use crate::semaphore::TrySemaphore;
use core::time::Duration;

/// A generic semaphore that can timeout.
///
/// # Safety
/// This trait is marked as unsafe to allow for safe code to rely on the standard semaphore contract.
/// [`Default`] implementations should initialize the count at 0.
pub unsafe trait TimeoutSemaphore: TrySemaphore {
    /// Blocks until can decrement or times out. Then returns [`true`] if decremented or [`false`] if timed out.
    fn wait_timeout(&self, timeout: Duration) -> bool;
}
