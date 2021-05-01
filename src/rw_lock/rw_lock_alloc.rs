use crate::rw_lock::{
    AsyncRwLock, AsyncTimeoutRwLock, CustomReadGuard, CustomRwLock, CustomWriteGuard,
    RawAsyncRwLock, RawAsyncTimeoutRwLock, TryRwLockSized,
};
use alloc::boxed::Box;
use core::future::Future;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::time::Duration;

/// The functions for [`AsyncRwLock`] that only work for sized types.
/// Separated to allow [`AsyncRwLock`] to be a trait object.
pub trait AsyncRwLockSized<'a>: Sized + AsyncRwLock<'a> + TryRwLockSized<'a> {
    /// Reads from the lock and runs `func` on the result asynchronously
    ///
    /// ## Implementation
    /// Should be overwritten by implementors if can be more optimal than
    /// creating a guard
    fn read_async_func<F>(
        &'a self,
        func: impl FnOnce(&Self::Item) -> F + 'a,
    ) -> Pin<Box<dyn Future<Output = F::Output> + 'a>>
    where
        F: Future,
    {
        Box::pin(async move { func(self.read_async().await.deref()).await })
    }

    /// Writes to the lock and runs `func` on the result asynchronously
    ///
    /// ## Implementation
    /// Should be overwritten by implementors if can be more optimal than
    /// creating a guard
    fn write_async_func<F>(
        &'a self,
        func: impl FnOnce(&mut Self::Item) -> F + 'a,
    ) -> Pin<Box<dyn Future<Output = F::Output> + 'a>>
    where
        F: Future,
    {
        Box::pin(async move { func(self.write_async().await.deref_mut()).await })
    }
}

/// The functions for [`AsyncTimeoutRwLock`] that only work for sized types.
/// Separated to allow [`AsyncTimeoutRwLock`] to be a trait object.
pub trait AsyncTimeoutRwLockSized<'a>:
    Sized + AsyncTimeoutRwLock<'a> + AsyncRwLockSized<'a>
{
    /// Reads form the lock with a timeout running func on the result
    /// asynchronously
    ///
    /// ## Implementation
    /// Should be overwritten by implementors if can be more optimal than
    /// creating a guard
    fn read_timeout_async_func<F>(
        &'a self,
        timeout: Duration,
        func: impl FnOnce(Option<&Self::Item>) -> F + 'a,
    ) -> Pin<Box<dyn Future<Output = F::Output> + 'a>>
    where
        F: Future,
    {
        Box::pin(async move {
            match self.read_timeout_async(timeout).await {
                None => func(None).await,
                Some(guard) => func(Some(guard.deref())).await,
            }
        })
    }

    /// Writes to the lock with a timeout running func on the result
    /// asynchronously
    ///
    /// ## Implementation
    /// Should be overwritten by implementors if can be more optimal than
    /// creating a guard
    fn write_timeout_async_func<F>(
        &'a self,
        timeout: Duration,
        func: impl FnOnce(Option<&mut Self::Item>) -> F + 'a,
    ) -> Pin<Box<dyn Future<Output = F::Output> + 'a>>
    where
        F: Future,
    {
        Box::pin(async move {
            match self.write_timeout_async(timeout).await {
                None => func(None).await,
                Some(mut guard) => func(Some(guard.deref_mut())).await,
            }
        })
    }
}

impl<'a, T, R> AsyncRwLock<'a> for CustomRwLock<T, R>
where
    T: 'a,
    R: RawAsyncRwLock + 'a,
{
    type AsyncReadGuard = CustomReadGuard<'a, T, R>;
    type AsyncWriteGuard = CustomWriteGuard<'a, T, R>;
    type ReadFuture = Pin<Box<dyn Future<Output = Self::AsyncReadGuard> + 'a>>;
    type WriteFuture = Pin<Box<dyn Future<Output = Self::AsyncWriteGuard> + 'a>>;

    fn read_async(&'a self) -> Self::ReadFuture {
        Box::pin(async move {
            self.raw_lock.add_reader_async().await;
            CustomReadGuard { lock: self }
        })
    }

    fn write_async(&'a self) -> Self::WriteFuture {
        Box::pin(async move {
            self.raw_lock.add_writer_async().await;
            CustomWriteGuard { lock: self }
        })
    }
}
impl<'a, T, R> AsyncRwLockSized<'a> for CustomRwLock<T, R>
where
    T: 'a,
    R: RawAsyncRwLock + 'a,
{
    fn read_async_func<F>(
        &'a self,
        func: impl FnOnce(&Self::Item) -> F + 'a,
    ) -> Pin<Box<dyn Future<Output = <F as Future>::Output> + 'a>>
    where
        F: Future,
    {
        Box::pin(async move {
            self.raw_lock.add_reader_async().await;
            unsafe {
                let out = func(&*self.data.get()).await;
                self.raw_lock.remove_reader();
                out
            }
        })
    }

    fn write_async_func<F>(
        &'a self,
        func: impl FnOnce(&mut Self::Item) -> F + 'a,
    ) -> Pin<Box<dyn Future<Output = <F as Future>::Output> + 'a>>
    where
        F: Future,
    {
        Box::pin(async move {
            self.raw_lock.add_writer_async().await;
            unsafe {
                let out = func(&mut *self.data.get()).await;
                self.raw_lock.remove_writer();
                out
            }
        })
    }
}
impl<'a, T, R> AsyncTimeoutRwLock<'a> for CustomRwLock<T, R>
where
    T: 'a,
    R: RawAsyncTimeoutRwLock + 'a,
{
    type ReadTimeoutFuture = Pin<Box<dyn Future<Output = Option<Self::AsyncReadGuard>> + 'a>>;
    type WriteTimeoutFuture = Pin<Box<dyn Future<Output = Option<Self::AsyncWriteGuard>> + 'a>>;

    fn read_timeout_async(&'a self, timeout: Duration) -> Self::ReadTimeoutFuture {
        Box::pin(async move {
            match self.raw_lock.add_reader_timeout_async(timeout).await {
                true => Some(CustomReadGuard { lock: self }),
                false => None,
            }
        })
    }

    fn write_timeout_async(&'a self, timeout: Duration) -> Self::WriteTimeoutFuture {
        Box::pin(async move {
            match self.raw_lock.add_writer_timeout_async(timeout).await {
                true => Some(CustomWriteGuard { lock: self }),
                false => None,
            }
        })
    }
}
impl<'a, T, R> AsyncTimeoutRwLockSized<'a> for CustomRwLock<T, R>
where
    T: 'a,
    R: RawAsyncTimeoutRwLock + 'a,
{
    fn read_timeout_async_func<F>(
        &'a self,
        timeout: Duration,
        func: impl FnOnce(Option<&Self::Item>) -> F + 'a,
    ) -> Pin<Box<dyn Future<Output = <F as Future>::Output> + 'a>>
    where
        F: Future,
    {
        Box::pin(async move {
            match self.raw_lock.add_reader_timeout_async(timeout).await {
                true => unsafe {
                    let out = func(Some(&*self.data.get())).await;
                    self.raw_lock.remove_reader();
                    out
                },
                false => func(None).await,
            }
        })
    }

    fn write_timeout_async_func<F>(
        &'a self,
        timeout: Duration,
        func: impl FnOnce(Option<&mut Self::Item>) -> F + 'a,
    ) -> Pin<Box<dyn Future<Output = <F as Future>::Output> + 'a>>
    where
        F: Future,
    {
        Box::pin(async move {
            match self.raw_lock.add_writer_timeout_async(timeout).await {
                true => unsafe {
                    let out = func(Some(&mut *self.data.get())).await;
                    self.raw_lock.remove_writer();
                    out
                },
                false => func(None).await,
            }
        })
    }
}
