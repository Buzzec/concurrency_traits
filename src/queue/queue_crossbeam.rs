use crate::queue::TryQueue;
use crossbeam::queue::{ArrayQueue, SegQueue};

impl<T> TryQueue for ArrayQueue<T> {
    type Item = T;

    fn try_push(&self, value: Self::Item) -> Result<(), Self::Item> {
        self.push(value)
    }

    fn try_pop(&self) -> Option<Self::Item> {
        self.pop()
    }

    fn clear(&self) {
        while self.pop().is_some() {}
    }
}
impl<T> TryQueue for SegQueue<T> {
    type Item = T;

    fn try_push(&self, value: Self::Item) -> Result<(), Self::Item> {
        self.push(value);
        Ok(())
    }

    fn try_pop(&self) -> Option<Self::Item> {
        self.pop()
    }

    fn clear(&self) {
        while self.pop().is_some() {}
    }
}
