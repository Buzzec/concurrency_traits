use crate::rw_lock::RawTryRwLock;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// A custom rw lock that can be built from any [`RawTryRwLock`] variant
#[derive(Debug)]
pub struct CustomRwLock<T, R> {
    pub(super) data: UnsafeCell<T>,
    pub(super) raw_lock: R,
}
impl<T, R> CustomRwLock<T, R> {
    /// Creates a lock from a [`RawTryRwLock`] variant
    pub fn from_raw(raw_lock: R, data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            raw_lock,
        }
    }

    /// Creates a lock using thr [`RawTryRwLock`] variant's default
    /// implementation
    pub fn new(data: T) -> Self
    where
        R: Default,
    {
        Self::from_raw(R::default(), data)
    }
}
unsafe impl<T, R> Sync for CustomRwLock<T, R>
where
    T: Sync,
    R: Sync,
{
}

/// The read guard for [`CustomRwLock`]
#[derive(Debug)]
pub struct CustomReadGuard<'a, T, R>
where
    R: RawTryRwLock,
{
    pub(super) lock: &'a CustomRwLock<T, R>,
}
impl<'a, T, R> Deref for CustomReadGuard<'a, T, R>
where
    R: RawTryRwLock,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}
impl<'a, T, R> Drop for CustomReadGuard<'a, T, R>
where
    R: RawTryRwLock,
{
    fn drop(&mut self) {
        unsafe { self.lock.raw_lock.remove_reader() }
    }
}

/// The write guard for [`CustomRwLock`]
#[derive(Debug)]
pub struct CustomWriteGuard<'a, T, R>
where
    R: RawTryRwLock,
{
    pub(super) lock: &'a CustomRwLock<T, R>,
}
impl<'a, T, R> Deref for CustomWriteGuard<'a, T, R>
where
    R: RawTryRwLock,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}
impl<'a, T, R> DerefMut for CustomWriteGuard<'a, T, R>
where
    R: RawTryRwLock,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}
impl<'a, T, R> Drop for CustomWriteGuard<'a, T, R>
where
    R: RawTryRwLock,
{
    fn drop(&mut self) {
        unsafe { self.lock.raw_lock.remove_writer() }
    }
}
