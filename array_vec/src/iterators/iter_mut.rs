use crate::ArrayVec;

/// Mutable iterator over `ArrayVec`. Alias for `slice::IterMut`.
pub type IterMut<'a, T> = <&'a mut [T] as IntoIterator>::IntoIter;

impl<'a, T, const N: usize> IntoIterator for &'a mut ArrayVec<T, N> {
    type IntoIter = IterMut<'a, T>;
    type Item = &'a mut T;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.as_mut_slice().iter_mut()
    }
}
