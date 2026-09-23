use crate::{ArrayVec, ArrayVecPopFront};

impl<T, const N: usize> IntoIterator for ArrayVec<T, N> {
    type IntoIter = IntoIter<T, N>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        // get_iter(self)
        let inner = ArrayVecPopFront::from_array_vec(self);
        IntoIter { inner }
    }
}

/// `IntoIter` for `ArrayVec`.
pub struct IntoIter<T, const N: usize> {
    inner: ArrayVecPopFront<T, N>,
}

const impl<T, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let leftover = self.inner.left();
        (leftover, Some(leftover))
    }
}

const impl<T, const N: usize> DoubleEndedIterator for IntoIter<T, N> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.pop()
    }
}
