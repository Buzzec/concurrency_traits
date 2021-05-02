#![allow(clippy::type_complexity)]

use crate::mutex::*;
use crate::queue::*;
use crate::rw_lock::*;
use crate::stack::*;
use core::future::Future;
use core::ops::{Deref, DerefMut};
use core::time::Duration;
use core::pin::Pin;
use core::mem::ManuallyDrop;
#[cfg(feature = "std")]
use std::panic::AssertUnwindSafe;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::rc::Rc;
#[cfg(feature = "alloc")]
use alloc::sync::Arc;
#[cfg(feature = "alloc")]
use alloc::borrow::Cow;


// TryMutex
macro_rules! impl_try_mutex_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T: ?Sized> TryMutex<'__a> for $impl_type where T: TryMutex<'__a>,
        {
            impl_try_mutex_deref!();
        }
    };
    (Sized $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> TryMutex<'__a> for $impl_type where T: TryMutex<'__a>,
        {
            impl_try_mutex_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> TryMutex<'__a> for $impl_type where T: TryMutex<'__a> + Clone,
        {
            impl_try_mutex_deref!();
        }
    };
    () => {
        type Item = T::Item;
        type Guard = T::Guard;

        #[inline]
        fn try_lock(&'__a self,) -> Option<Self::Guard> {
            self.deref().try_lock()
        }
    };
}
impl_try_mutex_deref!(&'a T, 'a);
impl_try_mutex_deref!(&'a mut T, 'a);
impl_try_mutex_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_try_mutex_deref!(Sized AssertUnwindSafe<T>);
#[cfg(feature = "alloc")]
impl_try_mutex_deref!(Rc<T>);
#[cfg(feature = "alloc")]
impl_try_mutex_deref!(Arc<T>);
#[cfg(feature = "alloc")]
impl_try_mutex_deref!(Box<T>);
#[cfg(feature = "alloc")]
impl_try_mutex_deref!(Clone Cow<'a, T>, 'a);
impl<'a, T> TryMutex<'a> for Pin<T> where T: Deref, T::Target: TryMutex<'a>{
    type Item = <T::Target as TryMutex<'a>>::Item;
    type Guard = <T::Target as TryMutex<'a>>::Guard;

    #[inline]
    fn try_lock(&'a self) -> Option<Self::Guard> {
        self.deref().try_lock()
    }
}
// Ensure can be trait object
impl<'a, I: ?Sized, G> dyn TryMutex<'a, Item = I, Guard = G> where G: DerefMut<Target = I> {}

// TryMutexSized
macro_rules! impl_try_mutex_sized_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> TryMutexSized<'__a> for $impl_type where T: TryMutexSized<'__a>,
        {
            impl_try_mutex_sized_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> TryMutexSized<'__a> for $impl_type where T: TryMutexSized<'__a> + Clone
        {
            impl_try_mutex_sized_deref!();
        }
    };
    () => {
        #[inline]
            fn try_lock_func<O>(&'__a self, func: impl FnOnce(Option<&mut Self::Item>) -> O) -> O {
                self.deref().try_lock_func(func)
            }
    };
}
impl_try_mutex_sized_deref!(&'a T, 'a);
impl_try_mutex_sized_deref!(&'a mut T, 'a);
// impl_try_mutex_sized_deref!(Pin<T>);
impl_try_mutex_sized_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_try_mutex_sized_deref!(AssertUnwindSafe<T>);
#[cfg(feature = "alloc")]
impl_try_mutex_sized_deref!(Rc<T>);
#[cfg(feature = "alloc")]
impl_try_mutex_sized_deref!(Arc<T>);
#[cfg(feature = "alloc")]
impl_try_mutex_sized_deref!(Box<T>);
#[cfg(feature = "alloc")]
impl_try_mutex_sized_deref!(Clone Cow<'a, T>, 'a);
impl<'a, T> TryMutexSized<'a> for Pin<T> where T: Deref, T::Target: TryMutexSized<'a>{
    #[inline]
    fn try_lock_func<O>(&'a self, func: impl FnOnce(Option<&mut Self::Item>) -> O) -> O {
        self.deref().try_lock_func(func)
    }
}

// Mutex
macro_rules! impl_mutex_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T: ?Sized> Mutex<'__a> for $impl_type where T: Mutex<'__a>,
        {
            impl_mutex_deref!();
        }
    };
    (Sized $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> Mutex<'__a> for $impl_type where T: Mutex<'__a>,
        {
            impl_mutex_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> Mutex<'__a> for $impl_type where T: Mutex<'__a> + Clone,
        {
            impl_mutex_deref!();
        }
    };
    () => {
        #[inline]
        fn lock(&'__a self) -> Self::Guard {
            self.deref().lock()
        }
    };
}
impl_mutex_deref!(&'a T, 'a);
impl_mutex_deref!(&'a mut T, 'a);
// impl_mutex_deref!(Sized Pin<T>);
impl_mutex_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_mutex_deref!(Sized AssertUnwindSafe<T>);
#[cfg(feature = "alloc")]
impl_mutex_deref!(Rc<T>);
#[cfg(feature = "alloc")]
impl_mutex_deref!(Arc<T>);
#[cfg(feature = "alloc")]
impl_mutex_deref!(Box<T>);
#[cfg(feature = "alloc")]
impl_mutex_deref!(Clone Cow<'a, T>, 'a);
impl<'a, T> Mutex<'a> for Pin<T> where T: Deref, T::Target: Mutex<'a>{
    #[inline]
    fn lock(&'a self) -> Self::Guard {
        self.deref().lock()
    }
}
// Ensure can be trait object
impl<'a, I: ?Sized, G> dyn Mutex<'a, Item = I, Guard = G> where G: DerefMut<Target = I> {}

// MutexSized
macro_rules! impl_mutex_sized_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> MutexSized<'__a> for $impl_type where T: MutexSized<'__a>,
        {
            impl_mutex_sized_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> MutexSized<'__a> for $impl_type where T: MutexSized<'__a> + Clone
        {
            impl_mutex_sized_deref!();
        }
    };
    () => {
        #[inline]
        fn lock_func<O>(&'__a self, func: impl FnOnce(&mut Self::Item) -> O) -> O {
            self.deref().lock_func(func)
        }
    };
}
impl_mutex_sized_deref!(&'a T, 'a);
impl_mutex_sized_deref!(&'a mut T, 'a);
// impl_mutex_sized_deref!(Pin<T>);
impl_mutex_sized_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_mutex_sized_deref!(AssertUnwindSafe<T>);
#[cfg(feature = "alloc")]
impl_mutex_sized_deref!(Rc<T>);
#[cfg(feature = "alloc")]
impl_mutex_sized_deref!(Arc<T>);
#[cfg(feature = "alloc")]
impl_mutex_sized_deref!(Box<T>);
#[cfg(feature = "alloc")]
impl_mutex_sized_deref!(Clone Cow<'a, T>, 'a);
impl<'a, T> MutexSized<'a> for Pin<T> where T: Deref, T::Target: MutexSized<'a>{
    #[inline]
    fn lock_func<O>(&'a self, func: impl FnOnce(&mut Self::Item) -> O) -> O {
        self.deref().lock_func(func)
    }
}

// AsyncMutex
macro_rules! impl_async_mutex_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T: ?Sized> AsyncMutex<'__a> for $impl_type where T: AsyncMutex<'__a>,
        {
            impl_async_mutex_deref!();
        }
    };
    (Sized $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> AsyncMutex<'__a> for $impl_type where T: AsyncMutex<'__a>,
        {
            impl_async_mutex_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> AsyncMutex<'__a> for $impl_type where T: AsyncMutex<'__a> + Clone,
        {
            impl_async_mutex_deref!();
        }
    };
    () => {
        type AsyncGuard = T::AsyncGuard;
        type LockFuture = T::LockFuture;

        #[inline]
        fn lock_async(&'__a self) -> Self::LockFuture {
            self.deref().lock_async()
        }
    };
}
impl_async_mutex_deref!(&'a T, 'a);
impl_async_mutex_deref!(&'a mut T, 'a);
// impl_async_mutex_deref!(Sized Pin<T>);
impl_async_mutex_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_async_mutex_deref!(Sized AssertUnwindSafe<T>);
#[cfg(feature = "alloc")]
impl_async_mutex_deref!(Rc<T>);
#[cfg(feature = "alloc")]
impl_async_mutex_deref!(Arc<T>);
#[cfg(feature = "alloc")]
impl_async_mutex_deref!(Box<T>);
#[cfg(feature = "alloc")]
impl_async_mutex_deref!(Clone Cow<'a, T>, 'a);
impl<'a, T> AsyncMutex<'a> for Pin<T> where T: Deref, T::Target: AsyncMutex<'a>{
    type AsyncGuard = <T::Target as AsyncMutex<'a>>::AsyncGuard;
    type LockFuture = <T::Target as AsyncMutex<'a>>::LockFuture;

    #[inline]
    fn lock_async(&'a self) -> Self::LockFuture {
        self.deref().lock_async()
    }
}
// Ensure can be trait object
impl<'a, I: ?Sized, G, AG, LF>
    dyn AsyncMutex<'a, Item = I, Guard = G, AsyncGuard = AG, LockFuture = LF>
where
    G: DerefMut<Target = I>,
    AG: DerefMut<Target = I>,
    LF: Future<Output = G>,
{
}

// TimeoutMutex
macro_rules! impl_timeout_mutex_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T: ?Sized> TimeoutMutex<'__a> for $impl_type where T: TimeoutMutex<'__a>,{
            impl_timeout_mutex_deref!();
        }
    };
    (Sized $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> TimeoutMutex<'__a> for $impl_type where T: TimeoutMutex<'__a>,{
            impl_timeout_mutex_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> TimeoutMutex<'__a> for $impl_type where T: TimeoutMutex<'__a> + Clone,{
            impl_timeout_mutex_deref!();
        }
    };
    () => {
        #[inline]
        fn lock_timeout(&'__a self, timeout: Duration) -> Option<Self::Guard> {
            self.deref().lock_timeout(timeout)
        }
    };
}
impl_timeout_mutex_deref!(&'a T, 'a);
impl_timeout_mutex_deref!(&'a mut T, 'a);
// impl_timeout_mutex_deref!(Sized Pin<T>);
impl_timeout_mutex_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_timeout_mutex_deref!(Sized AssertUnwindSafe<T>);
#[cfg(feature = "alloc")]
impl_timeout_mutex_deref!(Rc<T>);
#[cfg(feature = "alloc")]
impl_timeout_mutex_deref!(Arc<T>);
#[cfg(feature = "alloc")]
impl_timeout_mutex_deref!(Box<T>);
#[cfg(feature = "alloc")]
impl_timeout_mutex_deref!(Clone Cow<'a, T>, 'a);
impl<'a, T> TimeoutMutex<'a> for Pin<T> where T: Deref, T::Target: TimeoutMutex<'a>{
    #[inline]
    fn lock_timeout(&'a self, timeout: Duration) -> Option<Self::Guard> {
        self.deref().lock_timeout(timeout)
    }
}
// Ensure can be trait object
impl<'a, I: ?Sized, G> dyn TimeoutMutex<'a, Item = I, Guard = G> where G: DerefMut<Target = I> {}

// TimeoutMutexSized
macro_rules! impl_timeout_mutex_sized_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> TimeoutMutexSized<'__a> for $impl_type where T: TimeoutMutexSized<'__a>,
        {
            impl_timeout_mutex_sized_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> TimeoutMutexSized<'__a> for $impl_type where T: TimeoutMutexSized<'__a> + Clone
        {
            impl_timeout_mutex_sized_deref!();
        }
    };
    () => {
        #[inline]
        fn lock_timeout_func<O>(
            &'__a self,
            timeout: Duration,
            func: impl FnOnce(Option<&mut Self::Item>) -> O,
        ) -> O {
            self.deref().lock_timeout_func(timeout, func)
        }
    };
}
impl_timeout_mutex_sized_deref!(&'a T, 'a);
impl_timeout_mutex_sized_deref!(&'a mut T, 'a);
impl_timeout_mutex_sized_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_timeout_mutex_sized_deref!(AssertUnwindSafe<T>);
#[cfg(feature = "alloc")]
impl_timeout_mutex_sized_deref!(Rc<T>);
#[cfg(feature = "alloc")]
impl_timeout_mutex_sized_deref!(Arc<T>);
#[cfg(feature = "alloc")]
impl_timeout_mutex_sized_deref!(Box<T>);
#[cfg(feature = "alloc")]
impl_timeout_mutex_sized_deref!(Clone Cow<'a, T>, 'a);
impl<'a, T> TimeoutMutexSized<'a> for Pin<T> where T: Deref, T::Target: TimeoutMutexSized<'a>{
    #[inline]
    fn lock_timeout_func<O>(&'a self, timeout: Duration, func: impl FnOnce(Option<&mut Self::Item>) -> O) -> O {
        self.deref().lock_timeout_func(timeout, func)
    }
}

// AsyncTimeoutMutex
macro_rules! impl_async_timeout_mutex_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T: ?Sized> AsyncTimeoutMutex<'__a> for $impl_type where T: AsyncTimeoutMutex<'__a>,
        {
            impl_async_timeout_mutex_deref!();
        }
    };
    (Sized $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> AsyncTimeoutMutex<'__a> for $impl_type where T: AsyncTimeoutMutex<'__a>,
        {
            impl_async_timeout_mutex_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> AsyncTimeoutMutex<'__a> for $impl_type where T: AsyncTimeoutMutex<'__a> + Clone,
        {
            impl_async_timeout_mutex_deref!();
        }
    };
    () => {
        type LockTimeoutFuture = T::LockTimeoutFuture;

        #[inline]
        fn lock_timeout_async(&'__a self, timeout: Duration) -> Self::LockTimeoutFuture {
            self.deref().lock_timeout_async(timeout)
        }
    }
}
impl_async_timeout_mutex_deref!(&'a T, 'a);
impl_async_timeout_mutex_deref!(&'a mut T, 'a);
impl_async_timeout_mutex_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_async_timeout_mutex_deref!(Sized AssertUnwindSafe<T>);
#[cfg(feature = "alloc")]
impl_async_timeout_mutex_deref!(Rc<T>);
#[cfg(feature = "alloc")]
impl_async_timeout_mutex_deref!(Arc<T>);
#[cfg(feature = "alloc")]
impl_async_timeout_mutex_deref!(Box<T>);
#[cfg(feature = "alloc")]
impl_async_timeout_mutex_deref!(Clone Cow<'a, T>, 'a);
impl<'a, T> AsyncTimeoutMutex<'a> for Pin<T> where T: Deref, T::Target: AsyncTimeoutMutex<'a>{
    type LockTimeoutFuture = <T::Target as AsyncTimeoutMutex<'a>>::LockTimeoutFuture;

    #[inline]
    fn lock_timeout_async(&'a self, timeout: Duration) -> Self::LockTimeoutFuture {
        self.deref().lock_timeout_async(timeout)
    }
}
// Ensure can be trait object
impl<'a, I: ?Sized, G, AG, LF, LTF>
    dyn AsyncTimeoutMutex<
        'a,
        Item = I,
        Guard = G,
        AsyncGuard = AG,
        LockFuture = LF,
        LockTimeoutFuture = LTF,
    >
{
}

// TryRwLock
macro_rules! impl_try_rw_lock_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T: ?Sized> TryRwLock<'__a> for $impl_type where T: TryRwLock<'__a>,
        {
            impl_try_rw_lock_deref!();
        }
    };
    (Sized $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> TryRwLock<'__a> for $impl_type where T: TryRwLock<'__a>,
        {
            impl_try_rw_lock_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> TryRwLock<'__a> for $impl_type where T: TryRwLock<'__a> + Clone,
        {
            impl_try_rw_lock_deref!();
        }
    };
    () =>{
        type Item = T::Item;
        type ReadGuard = T::ReadGuard;
        type WriteGuard = T::WriteGuard;

        #[inline]
        fn try_read(&'__a self) -> Option<Self::ReadGuard> {
            self.deref().try_read()
        }

        #[inline]
        fn try_write(&'__a self) -> Option<Self::WriteGuard> {
            self.deref().try_write()
        }
    }
}
impl_try_rw_lock_deref!(&'a T, 'a);
impl_try_rw_lock_deref!(&'a mut T, 'a);
impl_try_rw_lock_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_try_rw_lock_deref!(Sized AssertUnwindSafe<T>);
#[cfg(feature = "alloc")]
impl_try_rw_lock_deref!(Rc<T>);
#[cfg(feature = "alloc")]
impl_try_rw_lock_deref!(Arc<T>);
#[cfg(feature = "alloc")]
impl_try_rw_lock_deref!(Box<T>);
#[cfg(feature = "alloc")]
impl_try_rw_lock_deref!(Clone Cow<'a, T>, 'a);
impl<'a, T> TryRwLock<'a> for Pin<T> where T: Deref, T::Target: TryRwLock<'a>{
    type Item = <T::Target as TryRwLock<'a>>::Item;
    type ReadGuard = <T::Target as TryRwLock<'a>>::ReadGuard;
    type WriteGuard = <T::Target as TryRwLock<'a>>::WriteGuard;

    #[inline]
    fn try_read(&'a self) -> Option<Self::ReadGuard> {
        self.deref().try_read()
    }

    #[inline]
    fn try_write(&'a self) -> Option<Self::WriteGuard> {
        self.deref().try_write()
    }
}
// Ensure can be trait object
impl<'a, I: ?Sized, RG, WG> dyn TryRwLock<'a, Item = I, ReadGuard = RG, WriteGuard = WG> {}

// TryRwLockSized
macro_rules! impl_try_rw_lock_sized_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> TryRwLockSized<'__a> for $impl_type where T: TryRwLockSized<'__a>,
        {
            impl_try_rw_lock_sized_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> TryRwLockSized<'__a> for $impl_type where T: TryRwLockSized<'__a> + Clone
        {
            impl_try_rw_lock_sized_deref!();
        }
    };
    () => {
        #[inline]
        fn try_read_func<O>(&'__a self, func: impl FnOnce(Option<&Self::Item>) -> O) -> O {
            self.deref().try_read_func(func)
        }

        #[inline]
        fn try_write_func<O>(&'__a self, func: impl FnOnce(Option<&mut Self::Item>) -> O) -> O {
            self.deref().try_write_func(func)
        }
    };
}
impl_try_rw_lock_sized_deref!(&'a T, 'a);
impl_try_rw_lock_sized_deref!(&'a mut T, 'a);
impl_try_rw_lock_sized_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_try_rw_lock_sized_deref!(AssertUnwindSafe<T>);
#[cfg(feature = "alloc")]
impl_try_rw_lock_sized_deref!(Rc<T>);
#[cfg(feature = "alloc")]
impl_try_rw_lock_sized_deref!(Arc<T>);
#[cfg(feature = "alloc")]
impl_try_rw_lock_sized_deref!(Box<T>);
#[cfg(feature = "alloc")]
impl_try_rw_lock_sized_deref!(Clone Cow<'a, T>, 'a);
impl<'a, T> TryRwLockSized<'a> for Pin<T> where T: Deref, T::Target: TryRwLockSized<'a>{
    #[inline]
    fn try_read_func<O>(&'a self, func: impl FnOnce(Option<&Self::Item>) -> O) -> O {
        self.deref().try_read_func(func)
    }

    fn try_write_func<O>(&'a self, func: impl FnOnce(Option<&mut Self::Item>) -> O) -> O {
        self.deref().try_write_func(func)
    }
}

// RwLock
macro_rules! impl_rw_lock_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T: ?Sized> RwLock<'__a> for $impl_type where T: RwLock<'__a>,
        {
            impl_rw_lock_deref!();
        }
    };
    (Sized $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> RwLock<'__a> for $impl_type where T: RwLock<'__a>,
        {
            impl_rw_lock_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> RwLock<'__a> for $impl_type where T: RwLock<'__a> + Clone,
        {
            impl_rw_lock_deref!();
        }
    };
    () =>{
        #[inline]
        fn read(&'__a self) -> Self::ReadGuard {
            self.deref().read()
        }

        #[inline]
        fn write(&'__a self) -> Self::WriteGuard {
            self.deref().write()
        }
    }
}
impl_rw_lock_deref!(&'a T, 'a);
impl_rw_lock_deref!(&'a mut T, 'a);
impl_rw_lock_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_rw_lock_deref!(Sized AssertUnwindSafe<T>);
#[cfg(feature = "alloc")]
impl_rw_lock_deref!(Rc<T>);
#[cfg(feature = "alloc")]
impl_rw_lock_deref!(Arc<T>);
#[cfg(feature = "alloc")]
impl_rw_lock_deref!(Box<T>);
#[cfg(feature = "alloc")]
impl_rw_lock_deref!(Clone Cow<'a, T>, 'a);
impl<'a, T> RwLock<'a> for Pin<T> where T: Deref, T::Target: RwLock<'a>{
    #[inline]
    fn read(&'a self) -> Self::ReadGuard {
        self.deref().read()
    }

    #[inline]
    fn write(&'a self) -> Self::WriteGuard {
        self.deref().write()
    }
}
// Ensure can be trait object
impl<'a, I: ?Sized, RG, WG> dyn RwLock<'a, Item = I, ReadGuard = RG, WriteGuard = WG> {}

// RwLockSized
macro_rules! impl_rw_lock_sized_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> RwLockSized<'__a> for $impl_type where T: RwLockSized<'__a>,
        {
            impl_rw_lock_sized_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> RwLockSized<'__a> for $impl_type where T: RwLockSized<'__a> + Clone
        {
            impl_rw_lock_sized_deref!();
        }
    };
    () => {
        #[inline]
        fn read_func<O>(&'__a self, func: impl FnOnce(&Self::Item) -> O) -> O {
            self.deref().read_func(func)
        }

        #[inline]
        fn write_func<O>(&'__a self, func: impl FnOnce(&mut Self::Item) -> O) -> O {
            self.deref().write_func(func)
        }
    };
}
impl_rw_lock_sized_deref!(&'a T, 'a);
impl_rw_lock_sized_deref!(&'a mut T, 'a);
impl_rw_lock_sized_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_rw_lock_sized_deref!(AssertUnwindSafe<T>);
#[cfg(feature = "alloc")]
impl_rw_lock_sized_deref!(Rc<T>);
#[cfg(feature = "alloc")]
impl_rw_lock_sized_deref!(Arc<T>);
#[cfg(feature = "alloc")]
impl_rw_lock_sized_deref!(Box<T>);
#[cfg(feature = "alloc")]
impl_rw_lock_sized_deref!(Clone Cow<'a, T>, 'a);
impl<'a, T> RwLockSized<'a> for Pin<T> where T: Deref, T::Target: RwLockSized<'a>{
    #[inline]
    fn read_func<O>(&'a self, func: impl FnOnce(&Self::Item) -> O) -> O {
        self.deref().read_func(func)
    }

    #[inline]
    fn write_func<O>(&'a self, func: impl FnOnce(&mut Self::Item) -> O) -> O {
        self.deref().write_func(func)
    }
}

// AsyncRwLock
macro_rules! impl_async_rw_lock_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T: ?Sized> AsyncRwLock<'__a> for $impl_type where T: AsyncRwLock<'__a>,
        {
            impl_async_rw_lock_deref!();
        }
    };
    (Sized $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> AsyncRwLock<'__a> for $impl_type where T: AsyncRwLock<'__a>,
        {
            impl_async_rw_lock_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> AsyncRwLock<'__a> for $impl_type where T: AsyncRwLock<'__a> + Clone,
        {
            impl_async_rw_lock_deref!();
        }
    };
    () =>{
        type AsyncReadGuard = T::AsyncReadGuard;
        type AsyncWriteGuard = T::AsyncWriteGuard;
        type ReadFuture = T::ReadFuture;
        type WriteFuture = T::WriteFuture;

        #[inline]
        fn read_async(&'__a self) -> Self::ReadFuture {
            self.deref().read_async()
        }

        #[inline]
        fn write_async(&'__a self) -> Self::WriteFuture {
            self.deref().write_async()
        }
    }
}
impl_async_rw_lock_deref!(&'a T, 'a);
impl_async_rw_lock_deref!(&'a mut T, 'a);
impl_async_rw_lock_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_async_rw_lock_deref!(Sized AssertUnwindSafe<T>);
#[cfg(feature = "alloc")]
impl_async_rw_lock_deref!(Rc<T>);
#[cfg(feature = "alloc")]
impl_async_rw_lock_deref!(Arc<T>);
#[cfg(feature = "alloc")]
impl_async_rw_lock_deref!(Box<T>);
#[cfg(feature = "alloc")]
impl_async_rw_lock_deref!(Clone Cow<'a, T>, 'a);
impl<'a, T> AsyncRwLock<'a> for Pin<T> where T: Deref, T::Target: AsyncRwLock<'a>{
    type AsyncReadGuard = <T::Target as AsyncRwLock<'a>>::AsyncReadGuard;
    type AsyncWriteGuard = <T::Target as AsyncRwLock<'a>>::AsyncWriteGuard;
    type ReadFuture = <T::Target as AsyncRwLock<'a>>::ReadFuture;
    type WriteFuture = <T::Target as AsyncRwLock<'a>>::WriteFuture;

    #[inline]
    fn read_async(&'a self) -> Self::ReadFuture {
        self.deref().read_async()
    }

    #[inline]
    fn write_async(&'a self) -> Self::WriteFuture {
        self.deref().write_async()
    }
}
// Ensure can be trait object
impl<'a, I: ?Sized, RG, WG, ARG, AWG, RF, WF>
    dyn AsyncRwLock<
        'a,
        Item = I,
        ReadGuard = RG,
        WriteGuard = WG,
        AsyncReadGuard = ARG,
        AsyncWriteGuard = AWG,
        ReadFuture = RF,
        WriteFuture = WF,
    >
{
}

// UpgradeRwLock
macro_rules! impl_upgrade_rw_lock_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T: ?Sized> UpgradeRwLock<'__a> for $impl_type where
            T: UpgradeRwLock<'__a>,
            <T as TryRwLock<'__a>>::ReadGuard: UpgradeReadGuard<
                '__a,
                Item = T::Item,
                WriteGuard = T::WriteGuard,
            >,
        {
            impl_upgrade_rw_lock_deref!();
        }
    };
    (Sized $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> UpgradeRwLock<'__a> for $impl_type where
            T: UpgradeRwLock<'__a>,
            <T as TryRwLock<'__a>>::ReadGuard: UpgradeReadGuard<
                '__a,
                Item = T::Item,
                WriteGuard = T::WriteGuard,
            >,
        {
            impl_upgrade_rw_lock_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> UpgradeRwLock<'__a> for $impl_type
        where
            T: UpgradeRwLock<'__a> + Clone,
            <T as TryRwLock<'__a>>::ReadGuard: UpgradeReadGuard<
                '__a,
                Item = T::Item,
                WriteGuard = T::WriteGuard,
            >,
        {
            impl_upgrade_rw_lock_deref!();
        }
    };
    () =>{}
}
impl_upgrade_rw_lock_deref!(&'a T, 'a);
impl_upgrade_rw_lock_deref!(&'a mut T, 'a);
impl_upgrade_rw_lock_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_upgrade_rw_lock_deref!(Sized AssertUnwindSafe<T>);
#[cfg(feature = "alloc")]
impl_upgrade_rw_lock_deref!(Rc<T>);
#[cfg(feature = "alloc")]
impl_upgrade_rw_lock_deref!(Arc<T>);
#[cfg(feature = "alloc")]
impl_upgrade_rw_lock_deref!(Box<T>);
#[cfg(feature = "alloc")]
impl_upgrade_rw_lock_deref!(Clone Cow<'a, T>, 'a);
impl<'a, T> UpgradeRwLock<'a> for Pin<T> where T: Deref, T::Target: UpgradeRwLock<'a>, <T::Target as TryRwLock<'a>>::ReadGuard: UpgradeReadGuard<'a, Item=<T::Target as TryRwLock<'a>>::Item, WriteGuard=<T::Target as TryRwLock<'a>>::WriteGuard>{}

// AsyncUpgradeRwLock
macro_rules! impl_async_upgrade_rw_lock_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T: ?Sized> AsyncUpgradeRwLock<'__a> for $impl_type where
            T: AsyncUpgradeRwLock<'__a>,
            <T as AsyncRwLock<'__a>>::AsyncReadGuard: AsyncUpgradeReadGuard<
                '__a,
                Item = T::Item,
                AsyncWriteGuard = T::AsyncWriteGuard,
            >,
        {
            impl_async_upgrade_rw_lock_deref!();
        }
    };
    (Sized $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> AsyncUpgradeRwLock<'__a> for $impl_type where
            T: AsyncUpgradeRwLock<'__a>,
            <T as AsyncRwLock<'__a>>::AsyncReadGuard: AsyncUpgradeReadGuard<
                '__a,
                Item = T::Item,
                AsyncWriteGuard = T::AsyncWriteGuard,
            >,
        {
            impl_async_upgrade_rw_lock_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> AsyncUpgradeRwLock<'__a> for $impl_type
        where
            T: AsyncUpgradeRwLock<'__a> + Clone,
            <T as AsyncRwLock<'__a>>::AsyncReadGuard: AsyncUpgradeReadGuard<
                '__a,
                Item = T::Item,
                AsyncWriteGuard = T::AsyncWriteGuard,
            >,
        {
            impl_async_upgrade_rw_lock_deref!();
        }
    };
    () =>{}
}
impl_async_upgrade_rw_lock_deref!(&'a T, 'a);
impl_async_upgrade_rw_lock_deref!(&'a mut T, 'a);
impl_async_upgrade_rw_lock_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_async_upgrade_rw_lock_deref!(Sized AssertUnwindSafe<T>);
#[cfg(feature = "alloc")]
impl_async_upgrade_rw_lock_deref!(Rc<T>);
#[cfg(feature = "alloc")]
impl_async_upgrade_rw_lock_deref!(Arc<T>);
#[cfg(feature = "alloc")]
impl_async_upgrade_rw_lock_deref!(Box<T>);
#[cfg(feature = "alloc")]
impl_async_upgrade_rw_lock_deref!(Clone Cow<'a, T>, 'a);
impl<'a, T> AsyncUpgradeRwLock<'a> for Pin<T> where T: Deref, T::Target: AsyncUpgradeRwLock<'a>, <T::Target as AsyncRwLock<'a>>::AsyncReadGuard: AsyncUpgradeReadGuard<'a, Item=<T::Target as TryRwLock<'a>>::Item, AsyncWriteGuard=<T::Target as AsyncRwLock<'a>>::AsyncWriteGuard>{}

// TimeoutRwLock
macro_rules! impl_timeout_rw_lock_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T: ?Sized> TimeoutRwLock<'__a> for $impl_type where T: TimeoutRwLock<'__a>,
        {
            impl_timeout_rw_lock_deref!();
        }
    };
    (Sized $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> TimeoutRwLock<'__a> for $impl_type where T: TimeoutRwLock<'__a>,
        {
            impl_timeout_rw_lock_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> TimeoutRwLock<'__a> for $impl_type where T: TimeoutRwLock<'__a> + Clone,
        {
            impl_timeout_rw_lock_deref!();
        }
    };
    () =>{
        #[inline]
        fn read_timeout(&'__a self, timeout: Duration) -> Option<Self::ReadGuard> {
            self.deref().read_timeout(timeout)
        }

        #[inline]
        fn write_timeout(&'__a self, timeout: Duration) -> Option<Self::WriteGuard> {
            self.deref().write_timeout(timeout)
        }
    }
}
impl_timeout_rw_lock_deref!(&'a T, 'a);
impl_timeout_rw_lock_deref!(&'a mut T, 'a);
impl_timeout_rw_lock_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_timeout_rw_lock_deref!(Sized AssertUnwindSafe<T>);
#[cfg(feature = "alloc")]
impl_timeout_rw_lock_deref!(Rc<T>);
#[cfg(feature = "alloc")]
impl_timeout_rw_lock_deref!(Arc<T>);
#[cfg(feature = "alloc")]
impl_timeout_rw_lock_deref!(Box<T>);
#[cfg(feature = "alloc")]
impl_timeout_rw_lock_deref!(Clone Cow<'a, T>, 'a);
impl<'a, T> TimeoutRwLock<'a> for Pin<T> where T: Deref, T::Target: TimeoutRwLock<'a>{
    #[inline]
    fn read_timeout(&'a self, timeout: Duration) -> Option<Self::ReadGuard> {
        self.deref().read_timeout(timeout)
    }

    #[inline]
    fn write_timeout(&'a self, timeout: Duration) -> Option<Self::WriteGuard> {
        self.deref().write_timeout(timeout)
    }
}
// Ensure can be trait object
impl<'a, I: ?Sized, RG, WG> dyn TimeoutRwLock<'a, Item = I, ReadGuard = RG, WriteGuard = WG> {}

// TimeoutRwLockSized
macro_rules! impl_timeout_rw_lock_sized_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> TimeoutRwLockSized<'__a> for $impl_type where T: TimeoutRwLockSized<'__a>,
        {
            impl_timeout_rw_lock_sized_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> TimeoutRwLockSized<'__a> for $impl_type where T: TimeoutRwLockSized<'__a> + Clone
        {
            impl_timeout_rw_lock_sized_deref!();
        }
    };
    () => {
        #[inline]
        fn read_timeout_func<O>(
            &'__a self,
            timeout: Duration,
            func: impl FnOnce(Option<&Self::Item>) -> O,
        ) -> O {
            self.deref().read_timeout_func(timeout, func)
        }

        #[inline]
        fn write_timeout_func<O>(
            &'__a self,
            timeout: Duration,
            func: impl FnOnce(Option<&mut Self::Item>) -> O,
        ) -> O {
            self.deref().write_timeout_func(timeout, func)
        }
    };
}
impl_timeout_rw_lock_sized_deref!(&'a T, 'a);
impl_timeout_rw_lock_sized_deref!(&'a mut T, 'a);
impl_timeout_rw_lock_sized_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_timeout_rw_lock_sized_deref!(AssertUnwindSafe<T>);
#[cfg(feature = "alloc")]
impl_timeout_rw_lock_sized_deref!(Rc<T>);
#[cfg(feature = "alloc")]
impl_timeout_rw_lock_sized_deref!(Arc<T>);
#[cfg(feature = "alloc")]
impl_timeout_rw_lock_sized_deref!(Box<T>);
#[cfg(feature = "alloc")]
impl_timeout_rw_lock_sized_deref!(Clone Cow<'a, T>, 'a);
impl<'a, T> TimeoutRwLockSized<'a> for Pin<T> where T: Deref, T::Target: TimeoutRwLockSized<'a>{
    #[inline]
    fn read_timeout_func<O>(&'a self, timeout: Duration, func: impl FnOnce(Option<&Self::Item>) -> O) -> O {
        self.deref().read_timeout_func(timeout, func)
    }

    #[inline]
    fn write_timeout_func<O>(&'a self, timeout: Duration, func: impl FnOnce(Option<&mut Self::Item>) -> O) -> O {
        self.deref().write_timeout_func(timeout, func)
    }
}

// AsyncTimeoutRwLock
macro_rules! impl_async_timeout_rw_lock_deref {
    ($impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T: ?Sized> AsyncTimeoutRwLock<'__a> for $impl_type where T: AsyncTimeoutRwLock<'__a>,
        {
            impl_async_timeout_rw_lock_deref!();
        }
    };
    (Sized $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> AsyncTimeoutRwLock<'__a> for $impl_type where T: AsyncTimeoutRwLock<'__a>,
        {
            impl_async_timeout_rw_lock_deref!();
        }
    };
    (Clone $impl_type:ty $(, $lifetime:lifetime)*) => {
        impl<'__a, $($lifetime,)* T> AsyncTimeoutRwLock<'__a> for $impl_type where T: AsyncTimeoutRwLock<'__a> + Clone,
        {
            impl_async_timeout_rw_lock_deref!();
        }
    };
    () =>{
        type ReadTimeoutFuture = T::ReadTimeoutFuture;
        type WriteTimeoutFuture = T::WriteTimeoutFuture;

        #[inline]
        fn read_timeout_async(&'__a self, timeout: Duration) -> Self::ReadTimeoutFuture {
            self.deref().read_timeout_async(timeout)
        }

        #[inline]
        fn write_timeout_async(&'__a self, timeout: Duration) -> Self::WriteTimeoutFuture {
            self.deref().write_timeout_async(timeout)
        }
    }
}
impl_async_timeout_rw_lock_deref!(&'a T, 'a);
impl_async_timeout_rw_lock_deref!(&'a mut T, 'a);
impl_async_timeout_rw_lock_deref!(ManuallyDrop<T>);
#[cfg(feature = "std")]
impl_async_timeout_rw_lock_deref!(Sized AssertUnwindSafe<T>);
#[cfg(feature = "alloc")]
impl_async_timeout_rw_lock_deref!(Rc<T>);
#[cfg(feature = "alloc")]
impl_async_timeout_rw_lock_deref!(Arc<T>);
#[cfg(feature = "alloc")]
impl_async_timeout_rw_lock_deref!(Box<T>);
#[cfg(feature = "alloc")]
impl_async_timeout_rw_lock_deref!(Clone Cow<'a, T>, 'a);
impl<'a, T> AsyncTimeoutRwLock<'a> for Pin<T> where T: Deref, T::Target: AsyncTimeoutRwLock<'a>{
    type ReadTimeoutFuture = <T::Target as AsyncTimeoutRwLock<'a>>::ReadTimeoutFuture;
    type WriteTimeoutFuture = <T::Target as AsyncTimeoutRwLock<'a>>::WriteTimeoutFuture;

    #[inline]
    fn read_timeout_async(&'a self, timeout: Duration) -> Self::ReadTimeoutFuture {
        self.deref().read_timeout_async(timeout)
    }

    #[inline]
    fn write_timeout_async(&'a self, timeout: Duration) -> Self::WriteTimeoutFuture {
        self.deref().write_timeout_async(timeout)
    }
}
// Ensure can be trait object
impl<'a, I: ?Sized, RG, WG, ARG, AWG, RF, WF, RTF, WTF>
    dyn AsyncTimeoutRwLock<
        'a,
        Item = I,
        ReadGuard = RG,
        WriteGuard = WG,
        AsyncReadGuard = ARG,
        AsyncWriteGuard = AWG,
        ReadFuture = RF,
        WriteFuture = WF,
        ReadTimeoutFuture = RTF,
        WriteTimeoutFuture = WTF,
    >
{
}

// TryQueue
impl<T: ?Sized> TryQueue for T
where
    T: Deref,
    T::Target: TryQueue,
{
    type Item = <T::Target as TryQueue>::Item;

    #[inline]
    fn try_push(&self, value: Self::Item) -> Result<(), Self::Item> {
        self.deref().try_push(value)
    }

    #[inline]
    fn try_pop(&self) -> Option<Self::Item> {
        self.deref().try_pop()
    }

    #[inline]
    fn clear(&self) {
        self.deref().clear()
    }
}
// Ensure can be trait object
impl<T> dyn TryQueue<Item = T> {}

// Queue
impl<T: ?Sized> Queue for T
where
    T: Deref,
    T::Target: Queue,
{
    #[inline]
    fn push(&self, value: Self::Item) {
        self.deref().push(value)
    }

    #[inline]
    fn pop(&self) -> Self::Item {
        self.deref().pop()
    }
}
// Ensure can be trait object
impl<T> dyn Queue<Item = T> {}

// AsyncQueue
impl<T: ?Sized> AsyncQueue for T
where
    T: Deref,
    T::Target: AsyncQueue,
{
    type AsyncItem = <T::Target as AsyncQueue>::AsyncItem;
    type PushFuture = <T::Target as AsyncQueue>::PushFuture;
    type PopFuture = <T::Target as AsyncQueue>::PopFuture;

    #[inline]
    fn push_async(&self, value: Self::AsyncItem) -> Self::PushFuture {
        self.deref().push_async(value)
    }

    #[inline]
    fn pop_async(&self) -> Self::PopFuture {
        self.deref().pop_async()
    }
}
// Ensure can be trait object
impl<T, PushF, PopF> dyn AsyncQueue<AsyncItem = T, PushFuture = PushF, PopFuture = PopF> {}

// TryPrependQueue
impl<T: ?Sized> TryPrependQueue for T
where
    T: Deref,
    T::Target: TryPrependQueue,
{
    #[inline]
    fn try_push_front(&self, value: Self::Item) -> Result<(), Self::Item> {
        self.deref().try_push_front(value)
    }
}
// Ensure can be trait object
impl<T> dyn TryPrependQueue<Item = T> {}

// PrependQueue
impl<T: ?Sized> PrependQueue for T
where
    T: Deref,
    T::Target: PrependQueue,
{
    #[inline]
    fn push_front(&self, value: Self::Item) {
        self.deref().push_front(value)
    }
}
// Ensure can be trait object
impl<T> dyn PrependQueue<Item = T> {}

// AsyncPrependQueue
impl<T: ?Sized> AsyncPrependQueue for T
where
    T: Deref,
    T::Target: AsyncPrependQueue,
{
    type PushBackFuture = <T::Target as AsyncPrependQueue>::PushBackFuture;

    #[inline]
    fn push_front_async(&self, value: Self::AsyncItem) -> Self::PushBackFuture {
        self.deref().push_front_async(value)
    }
}
// Ensure can be trait object
impl<T, PushF, PopF, PrepF>
    dyn AsyncPrependQueue<
        AsyncItem = T,
        PushFuture = PushF,
        PopFuture = PopF,
        PushBackFuture = PrepF,
    >
{
}

// TryReverseQueue
impl<T: ?Sized> TryReverseQueue for T
where
    T: Deref,
    T::Target: TryReverseQueue,
{
    #[inline]
    fn try_pop_back(&self) -> Option<Self::Item> {
        self.deref().try_pop_back()
    }
}
// Ensure can be trait object
impl<T> dyn TryReverseQueue<Item = T> {}

// ReverseQueue
impl<T: ?Sized> ReverseQueue for T
where
    T: Deref,
    T::Target: ReverseQueue,
{
    #[inline]
    fn pop_back(&self) -> Self::Item {
        self.deref().pop_back()
    }
}
// Ensure can be trait object
impl<T> dyn ReverseQueue<Item = T> {}

// AsyncReverseQueue
impl<T: ?Sized> AsyncReverseQueue for T
where
    T: Deref,
    T::Target: AsyncReverseQueue,
{
    type PopBackFuture = <T::Target as AsyncReverseQueue>::PopBackFuture;

    #[inline]
    fn pop_back_async(&self) -> Self::PopBackFuture {
        self.deref().pop_back_async()
    }
}
// Ensure can be trait object
impl<T, PushF, PopF, PBF>
    dyn AsyncReverseQueue<AsyncItem = T, PushFuture = PushF, PopFuture = PopF, PopBackFuture = PBF>
{
}

// TryPeekQueue
impl<T: ?Sized> TryPeekQueue for T
where
    T: Deref,
    T::Target: TryPeekQueue,
{
    type Peeked = <T::Target as TryPeekQueue>::Peeked;

    #[inline]
    fn try_peek(&self) -> Option<Self::Peeked> {
        self.deref().try_peek()
    }
}
// Ensure can be trait object
impl<T, P> dyn TryPeekQueue<Item = T, Peeked = P> {}

// PeekQueue
impl<T: ?Sized> PeekQueue for T
where
    T: Deref,
    T::Target: PeekQueue,
{
    #[inline]
    fn peek(&self) -> Self::Peeked {
        self.deref().peek()
    }
}
// Ensure can be trait object
impl<T, P> dyn PeekQueue<Item = T, Peeked = P> {}

// AsyncPeekQueue
impl<T: ?Sized> AsyncPeekQueue for T
where
    T: Deref,
    T::Target: AsyncPeekQueue,
{
    type AsyncPeeked = <T::Target as AsyncPeekQueue>::AsyncPeeked;
    type PeekFuture = <T::Target as AsyncPeekQueue>::PeekFuture;

    #[inline]
    fn peek_async(&self) -> Self::PeekFuture {
        self.deref().peek_async()
    }
}
// Ensure can be trait object
impl<T, P, PushF, PopF, PeekF>
    dyn AsyncPeekQueue<
        AsyncItem = T,
        AsyncPeeked = P,
        PushFuture = PushF,
        PopFuture = PopF,
        PeekFuture = PeekF,
    >
{
}

// TryPeekReverseQueue
impl<T: ?Sized> TryPeekReverseQueue for T
where
    T: Deref,
    T::Target: TryPeekReverseQueue,
{
    #[inline]
    fn try_peek_back(&self) -> Option<Self::Peeked> {
        self.deref().try_peek_back()
    }
}
// Ensure can be trait object
impl<T, P> dyn TryPeekReverseQueue<Item = T, Peeked = P> {}

// PeekReverseQueue
impl<T: ?Sized> PeekReverseQueue for T
where
    T: Deref,
    T::Target: PeekReverseQueue,
{
    #[inline]
    fn peek_back(&self) -> Self::Peeked {
        self.deref().peek_back()
    }
}
// Ensure can be trait object
impl<T, P> dyn PeekReverseQueue<Item = T, Peeked = P> {}

// AsyncPeekReverseQueue
impl<T: ?Sized> AsyncPeekReverseQueue for T
where
    T: Deref,
    T::Target: AsyncPeekReverseQueue,
{
    type PeekBackFuture = <T::Target as AsyncPeekReverseQueue>::PeekBackFuture;

    fn peek_back_async(&self) -> Self::PeekBackFuture {
        self.deref().peek_back_async()
    }
}
// Ensure can be trait object
impl<T, P, PushF, PopF, PeekF, PopBF, PeekBF>
    dyn AsyncPeekReverseQueue<
        AsyncItem = T,
        AsyncPeeked = P,
        PushFuture = PushF,
        PopFuture = PopF,
        PeekFuture = PeekF,
        PopBackFuture = PopBF,
        PeekBackFuture = PeekBF,
    >
{
}

// TryDoubleEndedQueue
impl<T: ?Sized> TryDoubleEndedQueue for T
where
    T: Deref,
    T::Target: TryDoubleEndedQueue,
{
}
// Ensure can be trait object
impl<T> dyn TryDoubleEndedQueue<Item = T> {}

// DoubleEndedQueue
impl<T: ?Sized> DoubleEndedQueue for T
where
    T: Deref,
    T::Target: DoubleEndedQueue,
{
}
// Ensure can be trait object
impl<T> dyn DoubleEndedQueue<Item = T> {}

// AsyncDoubleEndedQueue
impl<T: ?Sized> AsyncDoubleEndedQueue for T
where
    T: Deref,
    T::Target: AsyncDoubleEndedQueue,
{
}
// Ensure can be trait object
impl<T, PushF, PopF, PushBF, PopBF>
    dyn AsyncDoubleEndedQueue<
        AsyncItem = T,
        PushFuture = PushF,
        PopFuture = PopF,
        PushBackFuture = PushBF,
        PopBackFuture = PopBF,
    >
{
}

// TryStack
impl<T: ?Sized> TryStack for T
where
    T: Deref,
    T::Target: TryStack,
{
    type Item = <T::Target as TryStack>::Item;

    #[inline]
    fn try_push(&self, value: Self::Item) -> Result<(), Self::Item> {
        self.deref().try_push(value)
    }

    #[inline]
    fn try_pop(&self) -> Option<Self::Item> {
        self.deref().try_pop()
    }
}
// Ensure can be trait object
impl<T> dyn TryStack<Item = T> {}

// Stack
impl<T: ?Sized> Stack for T
where
    T: Deref,
    T::Target: Stack,
{
    #[inline]
    fn push(&self, value: Self::Item) {
        self.deref().push(value)
    }

    #[inline]
    fn pop(&self) -> Self::Item {
        self.deref().pop()
    }
}
// Ensure can be trait object
impl<T> dyn Stack<Item = T> {}

// AsyncStack
impl<T: ?Sized> AsyncStack for T
where
    T: Deref,
    T::Target: AsyncStack,
{
    type PushFuture = <T::Target as AsyncStack>::PushFuture;
    type PopFuture = <T::Target as AsyncStack>::PopFuture;

    fn push_async(&self, value: Self::Item) -> Self::PushFuture {
        self.deref().push_async(value)
    }

    fn pop_async(&self) -> Self::PopFuture {
        self.deref().pop_async()
    }
}
// Ensure can be trait object
impl<T, PushF, PopF> dyn AsyncStack<Item = T, PushFuture = PushF, PopFuture = PopF> {}
