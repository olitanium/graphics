//! Zip two arrays together and map with the given closure.
use core::{array, iter};

/// Zip two arrays together and map with the given closure.
/// for example
/// ```rust
/// # use utils::ArrayZipWith as _;
/// let x = [1, 2, 3];
/// let y = [4, 5, 6];
///
/// assert_eq!(x.zip_with(y, |a, b| a + b), [5, 7, 9])
/// ```
pub const trait ArrayZipWith<const N: usize> {
    /// The type of the element of the array.
    type Element;
    /// Zip two arrays together and map with the given closure.
    fn zip_with<U, V, F>(self, array2: [U; N], func: F) -> [V; N]
    where
        F: FnMut(Self::Element, U) -> V;
}

impl<const N: usize, T> ArrayZipWith<N> for [T; N] {
    type Element = T;

    #[inline]
    fn zip_with<U, V, F>(self, array2: [U; N], mut func: F) -> [V; N]
    where
        F: FnMut(T, U) -> V,
    {
        let mut zipped = iter::zip(self, array2);

        array::from_fn(|_| {
            // SAFETY: array1, array2 and the output are all arrays of length N, so there
            // will always be available values
            let (a, b) = unsafe { zipped.next().unwrap_unchecked() };
            func(a, b)
        })
    }
}
