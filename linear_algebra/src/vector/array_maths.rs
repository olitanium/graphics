macro_rules! elem_wise {
    ($trait:ident:: $fn:ident, $token:tt) => {
        #[inline]
        pub const fn $fn<const N: usize>(lhs: [f32; N], rhs: [f32; N]) -> [f32; N] {
            //core::array::from_fn(|index| core::ops::$trait::$fn(lhs[index], rhs[index]))

            let mut array = [0.0; N];
            let mut index = 0;

            while index < N {
                array[index] = lhs[index] $token rhs[index];
                index += 1;
            }

            array
        }
    };
}

pub const fn sum<const N: usize>(v1: [f32; N]) -> f32 {
    let mut output = 0.0;

    let mut index = 0;
    while index < N {
        output += v1[index];
        index += 1;
    }

    output
}

elem_wise!(Add::add, +);
elem_wise!(Sub::sub, -);
elem_wise!(Mul::mul, *);

pub const fn eq<const N: usize>(v1: [f32; N], v2: [f32; N]) -> bool {
    let diff = sub(v1, v2);
    is_zero(diff)
}

#[inline]
pub const fn dot<const N: usize>(v1: [f32; N], v2: [f32; N]) -> f32 {
    sum(mul(v1, v2))
}

#[inline]
pub const fn is_zero<const N: usize>(v1: [f32; N]) -> bool {
    const PRECISION_PER_DIMENSION: f32 = 0.001;
    dot(v1, v1) < const { PRECISION_PER_DIMENSION * N as f32 }
}
