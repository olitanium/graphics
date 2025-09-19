macro_rules! elem_wise {
    ($trait:ident:: $fn:ident) => {
        #[inline]
        pub fn $fn<const N: usize>(lhs: [f32; N], rhs: [f32; N]) -> [f32; N] {
            core::array::from_fn(|index| core::ops::$trait::$fn(lhs[index], rhs[index]))
        }
    };
}

elem_wise!(Add::add);
elem_wise!(Sub::sub);
elem_wise!(Mul::mul);

#[inline]
pub fn dot<const N: usize>(v1: [f32; N], v2: [f32; N]) -> f32 {
    mul(v1, v2).into_iter().sum()
}

#[inline]
pub fn is_zero(v1: &[f32]) -> bool {
    const EPSILON: f32 = 0.001;
    v1.iter().all(|x| x.abs() <= EPSILON)
}
