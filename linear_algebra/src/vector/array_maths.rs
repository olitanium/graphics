//! Module for performing array maths.

use core::marker::Destruct;
use core::mem;
use core::mem::MaybeUninit;
use core::ops::{Add, Mul, Sub};

use array_vec::ArrayVec;

/// Macro to generate trivially simple element wise operations.
macro_rules! elem_wise {
    ($trait:ident:: $fn:ident, $token:tt) => {
        #[inline]
        pub const fn $fn<const N: usize, T, U>(lhs: [T; N], rhs: [U; N]) -> [T; N] where T: [const] $trait<U, Output = T> {
            //core::array::from_fn(|index| core::ops::$trait::$fn(lhs[index], rhs[index])) // could literally be this

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
    // let mut index = 0;
    // let mut accum = T::default();
    // while let Some(value) = v1.get(index) {
    // accum = accum.clone() + value.clone();
    // index += 1;
    // }
    //
    // accum;

    let mut array = ArrayVec::from_array(v1);
    if let Some(mut accum) = array.pop() {
        while let Some(value) = array.pop() {
            accum = accum + value;
        }

        Some(accum)
    } else {
        None
    }
}

elem_wise!(Add::add, +);
elem_wise!(Sub::sub, -);
elem_wise!(Mul::mul, *);

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
pub const fn is_zero<const N: usize>(v1: [f32; N]) -> bool {
    const PRECISION_PER_DIMENSION: f32 = 0.001;
    let mut index = 0;
    while let Some(&value) = v1.get(index) {
        if value.abs() > PRECISION_PER_DIMENSION {
            return false;
        }

        index += 1;
    }
    true
}

/// Check whether all components are element-wise equal.
pub const fn eq<const N: usize>(v1: [f32; N], v2: [f32; N]) -> bool {
    // let diff = sub(v1, v2);
    // is_zero(diff)

    const PRECISION_PER_DIMENSION: f32 = 0.001;
    let mut index = 0;
    while let (Some(&value), Some(&value2)) = (v1.get(index), v2.get(index)) {
        if (value - value2).abs() > PRECISION_PER_DIMENSION {
            return false;
        }

        index += 1;
    }
    true
}

#[expect(clippy::inline_modules, reason = "this is a clippy bug")]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[expect(clippy::float_cmp, reason = "integer literals")]
    fn add() {
        assert_eq!(super::add([1.0, 2.0], [4.0, 8.0]), [5.0, 10.0]);
    }

    #[test]
    fn arr_sum() {
        assert_eq!(sum([1.0, 2.0, 3.0]), Some(6.0));
    }

    #[test]
    fn zero() {
        assert!(is_zero([0.0, 0.0, 0.0]));
        assert!(!is_zero([0.0, 0.0, 1.0]));
    }

    #[test]
    fn equal() {
        assert!(eq([1.0, 1.0], [1.0, 1.0]));
        assert!(!eq([0.0, 0.0], [1.0, 1.0]));
    }
}
