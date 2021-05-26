use crate::mutex::{Mutex, MutexSized, TimeoutMutex, TimeoutMutexSized, TryMutex, TryMutexSized};
use parking_lot::MutexGuard;
use std::time::Duration;

impl<'a, T: ?Sized> TryMutex<'a> for parking_lot::Mutex<T>
where
    T: 'a,
{
    type Item = T;
    type Guard = MutexGuard<'a, T>;

    fn try_lock(&'a self) -> Option<Self::Guard> {
        self.try_lock()
    }
}
impl<'a, T> TryMutexSized<'a> for parking_lot::Mutex<T> where T: 'a {}
impl<'a, T: ?Sized> Mutex<'a> for parking_lot::Mutex<T>
where
    T: 'a,
{
    fn lock(&'a self) -> Self::Guard {
        self.lock()
    }
}
impl<'a, T> MutexSized<'a> for parking_lot::Mutex<T> where T: 'a {}
impl<'a, T: ?Sized> TimeoutMutex<'a> for parking_lot::Mutex<T>
where
    T: 'a,
{
    fn lock_timeout(&'a self, timeout: Duration) -> Option<Self::Guard> {
        self.try_lock_for(timeout)
    }
}
impl<'a, T> TimeoutMutexSized<'a> for parking_lot::Mutex<T> where T: 'a {}
