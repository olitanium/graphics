//! Zip two arrays together and map with the given closure.
#![no_std]
#![feature(const_trait_impl)]
#![expect(incomplete_features, reason = "nightly used, risks assessed")]
#![feature(min_generic_const_args)]
#![feature(macroless_generic_const_args)]
#![feature(const_destruct)]
#![feature(const_array)]
#![feature(const_closures)]

use core::array;
use core::marker::Destruct;

use array_vec::ArrayVecPopFront;
/// Zip two arrays together and map with the given closure.
/// for example
/// ```rust
/// # use zip_with::ArrayZipWith as _;
/// let x = [1_usize, 2, 3];
/// let y = [4, 5, 6];
///
/// assert_eq!(x.zip_with(y, |a, b| a + b), [5, 7, 9])
/// ```
pub const trait ArrayZipWith {
    /// The type of the element of the array.
    type const LEN: usize;

    type Element;
    /// Length of the output (and input) arrays.
    /// Zip two arrays together and map with the given closure.
    fn zip_with<U, V, F>(self, array2: [U; Self::LEN], func: F) -> [V; Self::LEN]
    where
        F: [const] FnMut(Self::Element, U) -> V + [const] Destruct,
        Self::Element: [const] Destruct,
        U: [const] Destruct,
        V: [const] Destruct;
}

const impl<const N: usize, T> ArrayZipWith for [T; N] {
    type Element = T;

    type const LEN: usize = N;

    #[inline]
    fn zip_with<U, V, F>(self, array2: [U; N], mut func: F) -> [V; N]
    where
        F: [const] FnMut(T, U) -> V + [const] Destruct,
        T: [const] Destruct,
        U: [const] Destruct,
        V: [const] Destruct,
    {
        let mut array1 = ArrayVecPopFront::new(self);
        let mut array2 = ArrayVecPopFront::new(array2);

        array::from_fn(const |_| {
            func(
                // SAFETY: equal sized arrays, will be empty exactly as the function stops calling.
                unsafe { array1.pop_front().unwrap_unchecked() },
                // SAFETY: as above.
                unsafe { array2.pop_front().unwrap_unchecked() },
            )
        })
    }
}
