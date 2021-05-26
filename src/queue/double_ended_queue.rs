#[cfg(feature = "alloc")]
use crate::queue::{AsyncPrependQueue, AsyncReverseQueue};
use crate::queue::{PrependQueue, ReverseQueue, TryPrependQueue, TryReverseQueue};
/// A queue that can try to be written and read from both ends
pub trait TryDoubleEndedQueue: TryPrependQueue + TryReverseQueue {}
/// A queue that can be written and read from both ends
pub trait DoubleEndedQueue: PrependQueue + ReverseQueue + TryDoubleEndedQueue {}
/// An async queue that can be written and read from both ends
#[cfg(feature = "alloc")]
pub trait AsyncDoubleEndedQueue: AsyncPrependQueue + AsyncReverseQueue {}
