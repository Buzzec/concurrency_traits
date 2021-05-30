/// A generic semaphore that has try operations.
///
/// # Safety
/// This trait is marked as unsafe to allow for safe code to rely on the standard semaphore contract.
/// [`Default`] implementations should initialize the count at 0.
pub unsafe trait TrySemaphore: Default {
    /// Returns [`true`] if can decrement and did, or [`false`] if cannot.
    fn try_wait(&self) -> bool;
    /// Increments the counter.
    fn signal(&self);
}
