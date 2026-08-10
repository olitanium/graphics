use core::mem::MaybeUninit;

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

// fn get_iter<T, const N: usize>(vec: ArrayVec<T, N>) -> IntoIter<T, N> {
// let (len, array) = vec.into_raw_parts();
// SAFETY: taking `len` ensures that only init values are assumed init
// array
// .into_iter()
// .take(len)
// .map(|init| unsafe { init.assume_init() })
// }

// pub type IntoIter<T, const N: usize> = core::iter::Map<
// core::iter::Take<core::array::IntoIter<MaybeUninit<T>, N>>,
// fn(MaybeUninit<T>) -> T,
// >;

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
}

const impl<T, const N: usize> DoubleEndedIterator for IntoIter<T, N> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.pop()
    }
}
