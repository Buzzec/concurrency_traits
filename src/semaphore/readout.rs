use crate::semaphore::TrySemaphore;

/// A semaphore who's count can be read.
pub trait ReadoutSemaphore: TrySemaphore {
    /// The type of the count read from this semaphore.
    type Count;

    /// The count associated with this semaphore.
    fn count(&self) -> Self::Count;
}
