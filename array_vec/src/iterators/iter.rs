use std::iter::FusedIterator;
use std::mem::MaybeUninit;

use crate::ArrayVec;

pub struct Iter<'a, T, const N: usize> {
    pub(super) vec: &'a ArrayVec<T, N>,
    pub(super) next_index: usize,
    pub(super) end_index: usize,
}

impl<'a, T, const N: usize> IntoIterator for &'a ArrayVec<T, N> {
    type IntoIter = Iter<'a, T, N>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            vec: self,
            next_index: 0,
            end_index: self.len,
        }
    }
}

impl<'a, T, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index < self.end_index {
            let output = unsafe { MaybeUninit::assume_init_ref(&self.vec.array[self.next_index]) };
            self.next_index += 1;
            Some(output)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // 0123456789
        // xx!2345678

        // 9 - 2 + 1 = 8

        let difference = self.end_index - self.next_index + 1;
        (difference, Some(difference))
    }
}

impl<'a, T, const N: usize> DoubleEndedIterator for Iter<'a, T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.next_index < self.end_index {
            self.end_index -= 1;
            let output = unsafe { MaybeUninit::assume_init_ref(&self.vec.array[self.end_index]) };
            Some(output)
        } else {
            None
        }
    }
}

impl<'a, T, const N: usize> FusedIterator for Iter<'a, T, N> {}
impl<'a, T, const N: usize> ExactSizeIterator for Iter<'a, T, N> {}
