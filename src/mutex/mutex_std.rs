use std::sync;
use std::sync::{MutexGuard, TryLockError};
use crate::mutex::{TryMutex, TryMutexSized, Mutex, MutexSized};

impl<'a, T: ?Sized> TryMutex<'a> for sync::Mutex<T>
where
    T: 'a,
{
    type Item = T;
    type Guard = MutexGuard<'a, T>;

    fn try_lock(&'a self) -> Option<Self::Guard> {
        match self.try_lock() {
            Ok(guard) => Some(guard),
            Err(TryLockError::WouldBlock) => None,
            Err(TryLockError::Poisoned(error)) => panic!("Poison error: {}", error),
        }
    }
}
impl<'a, T> TryMutexSized<'a> for sync::Mutex<T> where T: 'a {}
impl<'a, T: ?Sized> Mutex<'a> for sync::Mutex<T>
where
    T: 'a,
{
    fn lock(&'a self) -> Self::Guard {
        match self.lock() {
            Ok(guard) => guard,
            Err(error) => panic!("Poison error: {}", error),
        }
    }
}
impl<'a, T> MutexSized<'a> for sync::Mutex<T> where T: 'a {}
