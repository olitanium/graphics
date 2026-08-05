//! A heap based version of the `MultiZip`, as an iterator it returns a
//! `Vec<T::Item>` for each item.
//!
//! It stops iterating once the first iterator in its collection has stopped.
//! The remainder of the iterators, in the order that they were added can be
//! retreived with `into_inner`.

/// See module docs.
///
/// # Example
///
/// ```
/// use multi_zip::HeapMultiZip;
/// let a = [1,2,3].as_slice();
/// let b = [4,5].as_slice();
/// let c = [6,7].as_slice();
///
/// let mut multi_zip = HeapMultiZip::new([a, b, c]);
///
/// assert_eq!(multi_zip.next(), Some(vec![&1, &4, &6]));
/// assert_eq!(multi_zip.next(), Some(vec![&2, &5, &7]));
/// assert_eq!(multi_zip.next(), None);
/// ```
#[derive(Debug)]
pub struct HeapMultiZip<T>(Vec<T>);

impl<T> Iterator for HeapMultiZip<T>
where
    T: Iterator,
{
    type Item = Vec<T::Item>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.iter_mut().map(Iterator::next).collect()
    }
}

impl<T> HeapMultiZip<T> {
    /// Create a new iterator which reads sequentially from the given input, and
    /// will yield each item one at a time.
    #[inline]
    pub fn new<M>(iters: M) -> Self
    where
        M: IntoIterator,
        <M as IntoIterator>::Item: IntoIterator<IntoIter = T>,
    {
        Self(iters.into_iter().map(IntoIterator::into_iter).collect())
    }

    /// Get the underlying iterators so that they can be each checked for
    /// exhaustion.
    #[must_use]
    #[inline]
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}
