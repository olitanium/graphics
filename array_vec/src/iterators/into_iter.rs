use std::mem::MaybeUninit;

use crate::ArrayVec;

impl<T, const N: usize> IntoIterator for ArrayVec<T, N> {
    type IntoIter = IntoIter<T, N>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        get_iter(self)
    }
}

fn assume_init_without_unsafe<T>(init: MaybeUninit<T>) -> T {
    // SAFETY: as in `get_iter`
    unsafe { init.assume_init() }
}

fn get_iter<T, const N: usize>(vec: ArrayVec<T, N>) -> IntoIter<T, N> {
    let (len, array) = vec.as_raw_parts();
    // SAFETY: taking `len` ensures that only init values are assumed_init
    array.into_iter().take(len).map(assume_init_without_unsafe)
}

pub type IntoIter<T, const N: usize> = std::iter::Map<std::iter::Take<std::array::IntoIter<MaybeUninit<T>, N>>, fn(MaybeUninit<T>) -> T>;