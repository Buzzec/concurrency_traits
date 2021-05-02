use crate::{TryRwLock, TryRwLockSized, RwLock, RwLockSized};
use std::sync;
use std::sync::{RwLockReadGuard, RwLockWriteGuard, TryLockError};

impl<'a, T: ?Sized> TryRwLock<'a> for sync::RwLock<T> where T: 'a{
    type Item = T;
    type ReadGuard = RwLockReadGuard<'a, T>;
    type WriteGuard = RwLockWriteGuard<'a, T>;

    fn try_read(&'a self) -> Option<Self::ReadGuard> where T: 'a {
        match self.try_read(){
            Ok(guard) => Some(guard),
            Err(TryLockError::WouldBlock) => None,
            Err(TryLockError::Poisoned(error)) => panic!("Poison Error: {}", error),
        }
    }

    fn try_write(&'a self) -> Option<Self::WriteGuard> where T: 'a {
        match self.try_write(){
            Ok(guard) => Some(guard),
            Err(TryLockError::WouldBlock) => None,
            Err(TryLockError::Poisoned(error)) => panic!("Poison Error: {}", error),
        }
    }
}
impl<'a, T> TryRwLockSized<'a> for sync::RwLock<T> where T: 'a{}
impl<'a, T> RwLock<'a> for sync::RwLock<T> where T: 'a{
    fn read(&'a self) -> Self::ReadGuard {
        match self.read() {
            Ok(guard) => guard,
            Err(error) => panic!("Poison Error: {}", error),
        }
    }

    fn write(&'a self) -> Self::WriteGuard {
        match self.write() {
            Ok(guard) => guard,
            Err(error) => panic!("Poison Error: {}", error),
        }
    }
}
impl<'a, T> RwLockSized<'a> for sync::RwLock<T> where T: 'a{}

