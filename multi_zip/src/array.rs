//! Zip a number of iterators together and return a fixed length array of
//! `T::Item`.
//!
//! It stops iterating once the first iterator in its collection has stopped.
//! The remainder of the iterators, in the order that they were added can be
//! retreived with `into_inner`.
use core::marker::Destruct;

use array_vec::ArrayVec;

/// See module docs.
///
/// # Example
///
/// ```
/// use multi_zip::MultiZip;
/// let a = [1,2,3].as_slice();
/// let b = [4,5].as_slice();
/// let c = [6,7].as_slice();
///
/// let mut multi_zip = MultiZip::new([a, b, c]);
///
/// assert_eq!(multi_zip.next(), Some([&1, &4, &6]));
/// assert_eq!(multi_zip.next(), Some([&2, &5, &7]));
/// assert_eq!(multi_zip.next(), None);
/// ```
#[derive(Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct MultiZip<T, const N: usize>([T; N]);

impl<T, const N: usize> Iterator for MultiZip<T, N>
where
    T: Iterator,
{
    type Item = [T::Item; N];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let out: Option<ArrayVec<T::Item, N>> = self.0.iter_mut().map(Iterator::next).collect();
        let inner = out?;
        inner.try_into().ok()
    }
}

impl<T: Iterator, const N: usize> MultiZip<T, N> {
    /// Create a new `MultiZip` iterator. The length of the input array
    /// determines the length of the outputs.
    #[inline]
    pub const fn new<I>(iters: [I; N]) -> Self
    where
        I: [const] IntoIterator<IntoIter = T> + [const] Destruct,
        T: [const] Destruct,
    {
        Self(iters.map(IntoIterator::into_iter))
    }

    /// Get the underlying iterators so that they can be each checked for
    /// exhaustion.
    #[must_use]
    #[inline]
    pub fn into_inner(self) -> [T; N] {
        self.0
    }
}
