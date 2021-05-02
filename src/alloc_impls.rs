use crate::mutex::*;
use crate::rw_lock::*;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::sync::Arc;
use core::future::Future;
use core::mem::ManuallyDrop;
use core::ops::Deref;
use core::pin::Pin;
use core::time::Duration;
#[cfg(feature = "std")]
use std::panic::AssertUnwindSafe;

// AsyncMutexSized
macro_rules! impl_async_mutex_sized_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> AsyncMutexSized<'__a> for $impl_type where T: AsyncMutexSized<'__a>,
        {
            impl_async_mutex_sized_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> AsyncMutexSized<'__a> for $impl_type where T: AsyncMutexSized<'__a> + Clone
        {
            impl_async_mutex_sized_deref!();
        }
    };
    () => {
        fn lock_async_func<F>(
            &'__a self,
            func: impl FnOnce(&mut Self::Item) -> F + '__a,
        ) -> Pin<Box<dyn Future<Output = <F as Future>::Output> + '__a>>
        where
            F: Future,
        {
            self.deref().lock_async_func(func)
        }
    };
}
impl_async_mutex_sized_deref!(&'a T, 'a);
impl_async_mutex_sized_deref!(&'a mut T, 'a);
// impl_async_mutex_sized_deref!(Pin<T>);
impl_async_mutex_sized_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_async_mutex_sized_deref!(AssertUnwindSafe<T>);
impl_async_mutex_sized_deref!(Rc<T>);
impl_async_mutex_sized_deref!(Arc<T>);
impl_async_mutex_sized_deref!(Box<T>);
impl_async_mutex_sized_deref!(Clone Cow<'a, T>, 'a);

// AsyncTimeoutMutexSized
macro_rules! impl_async_timeout_mutex_sized_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> AsyncTimeoutMutexSized<'__a> for $impl_type where T: AsyncTimeoutMutexSized<'__a>,
        {
            impl_async_timeout_mutex_sized_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> AsyncTimeoutMutexSized<'__a> for $impl_type where T: AsyncTimeoutMutexSized<'__a> + Clone
        {
            impl_async_timeout_mutex_sized_deref!();
        }
    };
    () => {
        #[inline]
        fn lock_timeout_async_func<F>(
            &'__a self,
            timeout: Duration,
            func: impl FnOnce(Option<&mut Self::Item>) -> F + '__a,
        ) -> Pin<Box<dyn Future<Output = <F as Future>::Output> + '__a>>
        where
            F: Future + '__a,
        {
            self.deref().lock_timeout_async_func(timeout, func)
        }
    };
}
impl_async_timeout_mutex_sized_deref!(&'a T, 'a);
impl_async_timeout_mutex_sized_deref!(&'a mut T, 'a);
// impl_async_timeout_mutex_sized_deref!(Pin<T>);
impl_async_timeout_mutex_sized_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_async_timeout_mutex_sized_deref!(AssertUnwindSafe<T>);
impl_async_timeout_mutex_sized_deref!(Rc<T>);
impl_async_timeout_mutex_sized_deref!(Arc<T>);
impl_async_timeout_mutex_sized_deref!(Box<T>);
impl_async_timeout_mutex_sized_deref!(Clone Cow<'a, T>, 'a);

// AsyncRwLockSized
macro_rules! impl_async_rw_lock_sized_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> AsyncRwLockSized<'__a> for $impl_type where T: AsyncRwLockSized<'__a>,
        {
            impl_async_rw_lock_sized_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> AsyncRwLockSized<'__a> for $impl_type where T: AsyncRwLockSized<'__a> + Clone
        {
            impl_async_rw_lock_sized_deref!();
        }
    };
    () => {
        #[inline]
        fn read_async_func<F>(
            &'__a self,
            func: impl FnOnce(&Self::Item) -> F + '__a,
        ) -> Pin<Box<dyn Future<Output = <F as Future>::Output> + '__a>>
        where
            F: Future,
        {
            self.deref().read_async_func(func)
        }

        #[inline]
        fn write_async_func<F>(
            &'__a self,
            func: impl FnOnce(&mut Self::Item) -> F + '__a,
        ) -> Pin<Box<dyn Future<Output = <F as Future>::Output> + '__a>>
        where
            F: Future,
        {
            self.deref().write_async_func(func)
        }
    };
}
impl_async_rw_lock_sized_deref!(&'a T, 'a);
impl_async_rw_lock_sized_deref!(&'a mut T, 'a);
// impl_async_rw_lock_sized_deref!(Pin<T>);
impl_async_rw_lock_sized_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_async_rw_lock_sized_deref!(AssertUnwindSafe<T>);
impl_async_rw_lock_sized_deref!(Rc<T>);
impl_async_rw_lock_sized_deref!(Arc<T>);
impl_async_rw_lock_sized_deref!(Box<T>);
impl_async_rw_lock_sized_deref!(Clone Cow<'a, T>, 'a);
