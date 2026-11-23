use crate::ArrayVec;

/// Shared iterator over `ArrayVec`. Alias for `slice::Iter`.
pub type Iter<'a, T> = <&'a [T] as IntoIterator>::IntoIter;

impl<'a, T, const N: usize> IntoIterator for &'a ArrayVec<T, N> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}
