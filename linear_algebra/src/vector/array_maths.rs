//! Module for performing array maths.

use core::marker::Destruct;
use core::mem;
use core::mem::MaybeUninit;
use core::ops::{Add, Div, Mul, Sub};

use array_vec::ArrayVec;
use float_eq::NearlyEq as _;

use crate::VectorPreferredFloat;

/// Macro to generate trivially simple element wise operations.
macro_rules! elem_wise {
    ($trait:ident:: $fn:ident, $token:tt) => {
        #[inline]
        pub const fn $fn<const N: usize, T, U>(lhs: [T; N], rhs: [U; N]) -> [T; N] where T: [const] $trait<U, Output = T> {
            //use utils::ArrayZipWith;
            //lhs.zip_with(rhs, core::ops::$trait::$fn) // cannot be used YET since not const

            let mut lhs: [MaybeUninit<T>; N] = MaybeUninit::new(lhs).transpose();
            let rhs: [MaybeUninit<U>; N] = MaybeUninit::new(rhs).transpose();

            let mut index = 0;
            while let Some((lhs_value_ref, rhs_value)) = Option::zip(lhs.get_mut(index), rhs.get(index)) {
                let lhs_value_maybeuninit = mem::replace(lhs_value_ref, MaybeUninit::zeroed());
                // SAFETY: The value will be init because it was init when input and has been unwritten.
                let lhs_value = unsafe { lhs_value_maybeuninit.assume_init() };
                // SAFETY: this is the only time that this reference will be read from, and the value will be init because it was init when input.
                // SAFETY: The value will be init because it was init when input and has been unwritten.
                let rhs_value = unsafe { rhs_value.assume_init_read() };

                lhs_value_ref.write(lhs_value $token rhs_value);
                index += 1;
            }

            // SAFETY: All `N` values will be written to from above.
            unsafe { MaybeUninit::array_assume_init(lhs) }
        }
    };
}
/// Sum the values in an array, without the `Sum` trait.
pub const fn sum<const N: usize, T>(v1: [T; N]) -> Option<T>
where
    T: [const] Add<Output = T> + [const] Destruct,
{
    let mut array = ArrayVec::from_array(v1);
    let out = array.pop().map(const |mut accum| {
        while let Some(value) = array.pop() {
            accum = accum + value;
        }
        accum
    });
    // #[expect(
    // clippy::mem_forget,
    // reason = "array drained, but ArrayVec needs drop. Forgetting the ArrayVed
    // removes [const] \ Drop requirement"
    // )]
    // mem::forget(array);
    out
}

elem_wise!(Add::add, +);
elem_wise!(Sub::sub, -);
elem_wise!(Mul::mul, *);
elem_wise!(Div::div, /);

/// Calculate the dot product, that is the sum of the element-wise products of
/// two cartesian vectors.
#[inline]
pub const fn dot<const N: usize, T>(v1: [T; N], v2: [T; N]) -> Option<T>
where
    T: [const] Add<Output = T>
        + [const] Mul<Output = T> // + [const] Default
        + [const] Destruct,
{
    sum(mul(v1, v2))
}

/// Check whether all components of the vector are zero (or very close).
#[inline]
pub const fn is_zero<const N: usize>(v1: &[VectorPreferredFloat; N]) -> bool {
    nearly_eq_f32(v1, &[0.0; N])
}

macro_rules! make_nearly_eq {
    ($name:ident($val:ty)) => {
        /// Check whether all components are element-wise equal.
        pub const fn $name<const N: usize>(v1: &[$val; N], v2: &[$val; N]) -> bool {
            let mut index = 0;
            while let (Some(value), Some(value2)) = (v1.get(index), v2.get(index)) {
                if !value.nearly_eq(value2, 0.005, 0.005) {
                    return false;
                }

                index += 1;
            }
            true
        }
    };
}

make_nearly_eq!(nearly_eq_f32(f32));
make_nearly_eq!(_nearly_eq_f64(f64));

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add() {
        assert_eq!(super::add([1.0, 2.0], [4.0, 8.0]), [5.0, 10.0]);
    }

    #[test]
    fn arr_sum() {
        assert_eq!(sum([1.0, 2.0, 3.0]), Some(6.0));
    }

    #[test]
    fn zero() {
        assert!(is_zero(&[0.0, 0.0, 0.0]));
        assert!(!is_zero(&[0.0, 0.0, 1.0]));
    }

    #[test]
    fn equal() {
        assert!(nearly_eq_f32(&[1.0, 1.0], &[1.0, 1.0]));
        assert!(!_nearly_eq_f64(&[0.0, 0.0], &[1.0, 1.0]));
    }
}
