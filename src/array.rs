use futures::stream::{FuturesOrdered, FuturesUnordered};

use futures::Stream;

pub trait FuturesArray<T: Future>: Stream<Item = T::Output> {
    fn push_fut(&mut self, item: T);

    fn is_empty(&self) -> bool;
}

impl<T: Future> FuturesArray<T> for FuturesOrdered<T> {
    fn push_fut(&mut self, item: T) {
        self.push_back(item);
    }

    fn is_empty(&self) -> bool {
        FuturesOrdered::is_empty(self)
    }
}

impl<T: Future> FuturesArray<T> for FuturesUnordered<T> {
    fn push_fut(&mut self, item: T) {
        self.push(item);
    }

    fn is_empty(&self) -> bool {
        FuturesUnordered::is_empty(self)
    }
}
