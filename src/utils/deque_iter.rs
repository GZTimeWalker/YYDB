use std::collections::VecDeque;

#[derive(Debug)]
pub struct DequeIterator<T> {
    cache: VecDeque<T>,
}

impl<T> DequeIterator<T> {
    pub fn new(cache: VecDeque<T>) -> Self {
        Self { cache }
    }
}

impl<T> Iterator for DequeIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.cache.pop_front()
    }
}
