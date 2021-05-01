use crate::mutex::*;
use crate::rw_lock::*;
use alloc::boxed::Box;
use core::future::Future;
use core::ops::Deref;
use core::pin::Pin;
use core::time::Duration;

// AsyncMutexSized
impl<'a, T> AsyncMutexSized<'a> for T
where
    T: Deref,
    T::Target: AsyncMutexSized<'a>,
{
    fn lock_async_func<F>(
        &'a self,
        func: impl FnOnce(&mut Self::Item) -> F + 'a,
    ) -> Pin<Box<dyn Future<Output = <F as Future>::Output> + 'a>>
    where
        F: Future,
    {
        self.deref().lock_async_func(func)
    }
}

// AsyncTimeoutMutexSized
impl<'a, T> AsyncTimeoutMutexSized<'a> for T
where
    T: Deref,
    T::Target: AsyncTimeoutMutexSized<'a>,
{
    #[inline]
    fn lock_timeout_async_func<F>(
        &'a self,
        timeout: Duration,
        func: impl FnOnce(Option<&mut Self::Item>) -> F + 'a,
    ) -> Pin<Box<dyn Future<Output = <F as Future>::Output> + 'a>>
    where
        F: Future,
    {
        self.deref().lock_timeout_async_func(timeout, func)
    }
}

// AsyncRwLockSized
impl<'a, T> AsyncRwLockSized<'a> for T
where
    T: Deref,
    T::Target: AsyncRwLockSized<'a>,
{
    #[inline]
    fn read_async_func<F>(
        &'a self,
        func: impl FnOnce(&Self::Item) -> F + 'a,
    ) -> Pin<Box<dyn Future<Output = <F as Future>::Output> + 'a>>
    where
        F: Future,
    {
        self.deref().read_async_func(func)
    }

    #[inline]
    fn write_async_func<F>(
        &'a self,
        func: impl FnOnce(&mut Self::Item) -> F + 'a,
    ) -> Pin<Box<dyn Future<Output = <F as Future>::Output> + 'a>>
    where
        F: Future,
    {
        self.deref().write_async_func(func)
    }
}
