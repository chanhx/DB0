//! This [`MultiPeekable`] implementation is directly inspired by `itertools` implementation.
use std::{collections::VecDeque, iter::Fuse};

pub struct MultiPeekable<I>
where
    I: Iterator,
{
    iter: Fuse<I>,
    buf: VecDeque<I::Item>,
}

pub trait MultiPeek: Iterator + Sized {
    fn multi_peekable(self) -> MultiPeekable<Self>;
}

impl<I: Iterator> MultiPeek for I {
    fn multi_peekable(self) -> MultiPeekable<I> {
        MultiPeekable {
            iter: self.fuse(),
            buf: VecDeque::new(),
        }
    }
}

impl<I: Iterator> MultiPeekable<I> {
    pub fn peek(&mut self) -> Option<&I::Item> {
        self.peek_nth(0)
    }

    pub fn peek_nth(&mut self, n: usize) -> Option<&I::Item> {
        let unbuffered_items = (n + 1).saturating_sub(self.buf.len());

        self.buf.extend(self.iter.by_ref().take(unbuffered_items));

        self.buf.get(n)
    }

    pub fn next_if(&mut self, func: impl FnOnce(&I::Item) -> bool) -> Option<I::Item> {
        match self.next()? {
            matched if func(&matched) => Some(matched),
            other => {
                self.buf.push_back(other);
                None
            }
        }
    }

    /// Advances the iterator by `n` elements if each element meets the condition
    pub fn advance_n_if_each(
        &mut self,
        n: usize,
        func: impl Fn((usize, &I::Item)) -> bool,
    ) -> Option<I::Item> {
        let mut items = self.by_ref().take(n).collect::<VecDeque<_>>();

        if items.len() < n || !items.iter().enumerate().all(|item| func(item)) {
            self.buf.append(&mut items);
            return None;
        }

        items.pop_back()
    }
}

impl<I> Iterator for MultiPeekable<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.buf.pop_front().or_else(|| self.iter.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (mut low, mut high) = self.iter.size_hint();
        low = low.saturating_add(self.buf.len());
        high = high.and_then(|elt| elt.checked_add(self.buf.len()));
        (low, high)
    }
}

// Same size
impl<I> ExactSizeIterator for MultiPeekable<I> where I: ExactSizeIterator {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peek() {
        let xs = [1, 2, 3];
        let mut iter = xs.iter().multi_peekable();

        // peek() lets us see into the future
        assert_eq!(iter.peek(), Some(&&1));
        assert_eq!(iter.next(), Some(&1));

        assert_eq!(iter.next(), Some(&2));

        // The iterator does not advance even if we `peek` multiple times
        assert_eq!(iter.peek(), Some(&&3));
        assert_eq!(iter.peek(), Some(&&3));

        assert_eq!(iter.next(), Some(&3));

        // After the iterator is finished, so is `peek()`
        assert_eq!(iter.peek(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn peek_nth() {
        let xs = [1, 2, 3];
        let mut iter = xs.iter().multi_peekable();

        // n starts from 0
        assert_eq!(iter.peek_nth(0), Some(&&1));
        assert_eq!(iter.peek_nth(1), Some(&&2));
        assert_eq!(iter.peek_nth(2), Some(&&3));

        // If n reaches the end, return None
        assert_eq!(iter.peek_nth(3), None);

        assert_eq!(iter.next(), Some(&1));

        // The iterator does not advance even if we `peek_nth` multiple times
        assert_eq!(iter.peek_nth(1), Some(&&3));
        assert_eq!(iter.peek_nth(1), Some(&&3));

        assert_eq!(iter.next(), Some(&2));
    }

    #[test]
    fn next_if() {
        let mut iter = (0..5).multi_peekable();

        assert_eq!(iter.next_if(|&x| x == 1), None);
        // The first item of the iterator is 0; consume it.
        assert_eq!(iter.next_if(|&x| x == 0), Some(0));
        // The next item returned is now 1, so `consume` will return `false`.
        assert_eq!(iter.next_if(|&x| x == 0), None);
        // `next_if` saves the value of the next item if it was not equal to `expected`.
        assert_eq!(iter.next(), Some(1));
    }

    #[test]
    fn advance_n_if_each() {
        let mut iter = (0..5).multi_peekable();

        assert_eq!(iter.advance_n_if_each(2, |(i, &num)| i == num), Some(1));
        assert_eq!(iter.next(), Some(2));

        assert_eq!(iter.advance_n_if_each(2, |(i, _)| i == 99), None);
        assert_eq!(iter.next(), Some(3));
    }
}
