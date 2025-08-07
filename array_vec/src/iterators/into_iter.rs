use std::iter::FusedIterator;
use std::mem::{self, MaybeUninit};

use crate::ArrayVec;

pub struct IntoIter<T, const N: usize> {
    pub(super) vec: ArrayVec<T, N>,
    pub(super) next_index: usize,
}

impl<T, const N: usize> IntoIterator for ArrayVec<T, N> {
    type IntoIter = IntoIter<T, N>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            vec: self,
            next_index: 0,
        }
    }
}

impl<T, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index < self.vec.len {
            let mut output = MaybeUninit::uninit();
            mem::swap(&mut output, &mut self.vec.array[self.next_index]);
            self.next_index += 1;

            let output = unsafe { MaybeUninit::assume_init(output) };

            Some(output)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // 0123456789
        // xx!2345678

        // 9 - 2 + 1 = 8

        let difference = self.vec.len - self.next_index + 1;
        (difference, Some(difference))
    }
}

impl<T, const N: usize> DoubleEndedIterator for IntoIter<T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.vec.pop_unsecure()
    }
}

impl<T, const N: usize> FusedIterator for IntoIter<T, N> {}
impl<T, const N: usize> ExactSizeIterator for IntoIter<T, N> {}
