use std::marker::Destruct;
use core::ops::{Add, Mul, Sub};

macro_rules! elem_wise {
    ($trait:ident:: $fn:ident, $token:tt) => {
        #[inline]
        pub const fn $fn<const N: usize, T>(mut lhs: [T; N], rhs: [T; N]) -> [T; N] where T: [const] $trait<Output = T> + [const] Clone + [const] Destruct {
            //core::array::from_fn(|index| core::ops::$trait::$fn(lhs[index], rhs[index]))

            /*let mut array = lhs;
            let mut index = 0;

            while index < N {
                array[index] = array[index] $token rhs[index];
                index += 1;
            }*/

            let mut index = 0;
            while let Some((lhs_value, rhs_value)) = Option::zip(lhs.get_mut(index), rhs.get(index)) {
                *lhs_value = lhs_value.clone() $token rhs_value.clone();
                index += 1;
            }

            lhs
        }
    };
}

pub const fn sum<
    const N: usize,
    T,
>(
    v1: [T; N],
) -> T where T: [const] Add<Output = T> + [const] Clone + [const] Destruct + [const] Default {
    // if N > 0 {
    // let mut output = v1[0];
    //
    // let mut index = 1;
    // while index < N {
    // output = output + v1[index];
    // index += 1;
    // }
    //
    // output
    // } else {
    // T::default()
    // }

    let mut index = 0;
    let mut accum = T::default();
    while let Some(value) = v1.get(index) {
        accum = accum.clone() + value.clone();
        index += 1;
    }

    accum
}

elem_wise!(Add::add, +);
elem_wise!(Sub::sub, -);
elem_wise!(Mul::mul, *);

#[inline]
pub const fn dot<
    const N: usize,
    T,
>(
    v1: [T; N],
    v2: [T; N],
) -> T where T: [const] Add<Output = T>
        + [const] Mul<Output = T>
        + [const] Clone
        + [const] Default
        + [const] Destruct {
    sum(mul(v1, v2))
}

#[inline]
pub const fn is_zero<const N: usize>(v1: [f32; N]) -> bool {
    const PRECISION_PER_DIMENSION: f32 = 0.001;
    dot(v1, v1) < const { PRECISION_PER_DIMENSION * N as f32 }
}

pub const fn eq<const N: usize>(v1: [f32; N], v2: [f32; N]) -> bool {
    let diff = sub(v1, v2);
    is_zero(diff)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sum() {
        assert_eq!(sum([1.0, 2.0, 3.0]), 6.0)
    }
}
