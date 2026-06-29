use std::ops::{Add, Mul, Sub};

macro_rules! elem_wise {
    ($trait:ident:: $fn:ident, $token:tt) => {
        #[inline]
        pub const fn $fn<const N: usize, T: [const] $trait<Output = T> + Copy>(lhs: [T; N], rhs: [T; N]) -> [T; N] {
            //core::array::from_fn(|index| core::ops::$trait::$fn(lhs[index], rhs[index]))

            let mut array = lhs;
            let mut index = 0;

            while index < N {
                array[index] = array[index] $token rhs[index];
                index += 1;
            }

            array
        }
    };
}

pub const fn sum<const N: usize, T: [const] Add<Output = T> + Copy + [const] Default>(
    v1: [T; N],
) -> T {
    if N > 0 {
        let mut output = v1[0];

        let mut index = 1;
        while index < N {
            output = output + v1[index];
            index += 1;
        }

        output
    } else {
        T::default()
    }
}

elem_wise!(Add::add, +);
elem_wise!(Sub::sub, -);
elem_wise!(Mul::mul, *);

#[inline]
pub const fn dot<
    const N: usize,
    T: [const] Add<Output = T> + [const] Mul<Output = T> + Copy + [const] Default,
>(
    v1: [T; N],
    v2: [T; N],
) -> T {
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
