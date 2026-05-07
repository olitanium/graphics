mod array_maths;

use core::array;
use core::ops::{Index, IndexMut};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Mul, MulAssign, SubAssign};
use std::slice::SliceIndex;

use super::matrix::Matrix;

mod unit_vector;
pub use unit_vector::UnitVector;

mod compact_unit;
pub use compact_unit::CompactUnitVector;

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct Vector<const N: usize>(pub(crate) [f32; N]);

impl<const N: usize> const Default for Vector<N> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<const N: usize> PartialEq for Vector<N> {
    fn eq(&self, other: &Self) -> bool {
        array_maths::eq(self.into_inner(), other.into_inner())
    }
}

impl<const N: usize> Vector<N> {
    pub const X: Self = Self::x();
    pub const ZERO: Self = Self::new([0.0; N]);

    const fn x() -> Self {
        let mut vec = Self::ZERO;
        vec.inner_mut()[0] = 1.0;
        vec
    }

    #[must_use]
    #[inline]
    pub const fn new(input: [f32; N]) -> Self {
        Self(input)
    }

    /// # Safety
    ///
    /// The vector must be have a magnitude of 1.0, or this may break future
    /// optimisations
    #[must_use]
    #[inline]
    pub const unsafe fn new_normal_unchecked(input: [f32; N]) -> UnitVector<N> {
        UnitVector::new_unchecked(input)
    }

    #[must_use]
    #[inline]
    pub const fn into_inner(self) -> [f32; N] {
        self.0
    }

    #[must_use]
    #[inline]
    pub const fn inner(&self) -> &[f32; N] {
        &self.0
    }

    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> usize {
        N
    }

    pub fn get<I: SliceIndex<[f32]>>(&self, index: I) -> Option<&<I as SliceIndex<[f32]>>::Output> {
        self.0.get(index)
    }

    #[must_use]
    #[inline]
    pub const fn inner_mut(&mut self) -> &mut [f32; N] {
        &mut self.0
    }

    #[inline]
    pub const fn iter_mut(&mut self) -> impl Iterator<Item = &mut f32> {
        self.inner_mut().iter_mut()
    }

    #[must_use]
    #[inline]
    pub fn map(self, op: impl Fn(f32) -> f32) -> Self {
        let array = self.into_inner().map(op);
        Self::new(array)
    }

    pub const fn magnitude_sq(self) -> f32 {
        self.dot(self)
    }

    pub fn magnitude(self) -> f32 {
        self.magnitude_sq().sqrt()
    }

    /// # Panics
    #[must_use]
    #[inline]
    pub fn normalize(self) -> UnitVector<N> {
        let Some(vec) = self.try_normalize() else {
            panic!("tried normalizing a zero vector");
        };
        vec
    }

    pub fn try_normalize(self) -> Option<UnitVector<N>> {
        let one_len = 1.0 / self.magnitude();
        one_len
            .is_finite()
            .then(|| unsafe { Self::new_normal_unchecked(self.0.map(|x| x * one_len)) })
    }

    #[inline]
    #[must_use]
    pub const fn is_zero(self) -> bool {
        array_maths::is_zero(self.into_inner())
    }

    #[inline]
    #[must_use]
    pub const fn dot(self, rhs: Self) -> f32 {
        array_maths::dot(self.into_inner(), rhs.into_inner())
    }

    #[inline]
    #[must_use]
    pub const fn from_to(from: Self, to: Self) -> Self {
        to.sub(from)
    }

    #[must_use]
    #[inline]
    pub const fn distance_squared(self, to: Self) -> f32 {
        let direction = Self::from_to(self, to);
        direction.dot(direction)
    }

    #[inline]
    #[must_use]
    pub const fn scale(mut self, scalar: f32) -> Self {
        // CONST-HACK
        // self.map(|elem| scalar * elem)

        let mut index = 0;
        while index < N {
            self.0[index] *= scalar;
            index += 1;
        }

        self
    }

    pub const fn flip(self) -> Self {
        // CONST-HACK
        // for x in self.inner_mut() {
        // x = -*x;
        // }
        //
        // self

        Vector::new([0.0; N]).sub(self)
    }

    pub const fn truncate<const M: usize>(self) -> Vector<M> {
        // CONST-HACK
        // let array = array::from_fn(|index|
        // self.0.get(index).copied().unwrap_or(0.0)); Vector::new(array)

        let mut array = [0.0; M];
        let mut index = 0;
        while let (Some(value), Some(dest)) = (self.0.get(index).copied(), array.get_mut(index)) {
            *dest = value;
            index += 1;
        }

        Vector::new(array)
    }

    pub const fn reflect(self, normal: UnitVector<N>) -> Self {
        let normal = normal.v();
        let projection = normal.dot(self);
        self.sub(normal.scale(2.0 * projection))
    }
}

impl Vector<3> {
    #[inline]
    #[must_use]
    pub const fn cross(self, rhs: Self) -> Self {
        let lhs = self.into_inner();
        let rhs = rhs.into_inner();
        let out = [
            lhs[1].mul_add(rhs[2], -(lhs[2] * rhs[1])), // lhs[1] * rhs[2] - lhs[2] * rhs[1],
            lhs[2].mul_add(rhs[0], -(lhs[0] * rhs[2])), // lhs[2] * rhs[0] - lhs[0] * rhs[2],
            lhs[0].mul_add(rhs[1], -(lhs[1] * rhs[0])), // lhs[0] * rhs[1] - lhs[1] * rhs[0],
        ];
        Self::new(out)
    }

    pub const fn homogeneous(self) -> Vector<4> {
        let this = self.into_inner();
        Vector::new([this[0], this[1], this[2], 1.0])
    }
}

macro_rules! impl_ops {
    ($trait:ident:: $fn:ident) => {
        use self::array_maths::$fn;
        impl<const N: usize> core::ops::$trait for Vector<N> {
            type Output = Self;

            #[inline]
            fn $fn(self, rhs: Self) -> Self::Output {
                Self::new($fn(self.into_inner(), rhs.into_inner()))
            }
        }

        impl<const N: usize> Vector<N> {
            pub const fn $fn(self, rhs: Self) -> Self {
                Self::new($crate::vector::array_maths::$fn(
                    self.into_inner(),
                    rhs.into_inner(),
                ))
            }
        }

        impl<const N: usize> core::ops::$trait<UnitVector<N>> for Vector<N> {
            type Output = Self;

            #[inline]
            fn $fn(self, rhs: UnitVector<N>) -> Self::Output {
                Self::Output::new($fn(self.into_inner(), rhs.0.into_inner()))
            }
        }

        impl<const N: usize> core::ops::$trait<Vector<N>> for UnitVector<N> {
            type Output = Vector<N>;

            #[inline]
            fn $fn(self, rhs: Vector<N>) -> Vector<N> {
                Vector::new($fn(self.0.into_inner(), rhs.into_inner()))
            }
        }
    };
}

impl_ops!(Add::add);
impl_ops!(Sub::sub);

impl<const N: usize> AddAssign for Vector<N> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<const N: usize> SubAssign for Vector<N> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<const N: usize> Mul<f32> for Vector<N> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        self.scale(rhs)
    }
}

impl<const N: usize> MulAssign<f32> for Vector<N> {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl<const N: usize> Mul<Vector<N>> for f32 {
    type Output = Vector<N>;

    fn mul(self, rhs: Vector<N>) -> Self::Output {
        rhs * self
    }
}

impl<const R: usize> From<Matrix<R, 1>> for Vector<R> {
    #[inline]
    fn from(value: Matrix<R, 1>) -> Self {
        value.into_inner()[0]
    }
}

impl<const N: usize> From<[f32; N]> for Vector<N> {
    #[inline]
    fn from(value: [f32; N]) -> Self {
        Self::new(value)
    }
}

impl<const N: usize> From<Vector<N>> for [f32; N] {
    #[inline]
    fn from(value: Vector<N>) -> Self {
        value.into_inner()
    }
}

impl<'a, const N: usize> TryFrom<&'a [f32]> for Vector<N> {
    type Error = <[f32; N] as TryFrom<&'a [f32]>>::Error;

    #[inline]
    fn try_from(value: &[f32]) -> Result<Self, Self::Error> {
        value.try_into().map(Self::new)
    }
}

impl<const N: usize> AsRef<[f32; N]> for Vector<N> {
    #[inline]
    fn as_ref(&self) -> &[f32; N] {
        self.inner()
    }
}

impl<const N: usize> AsRef<[f32]> for Vector<N> {
    #[inline]
    fn as_ref(&self) -> &[f32] {
        self.inner()
    }
}

impl<const N: usize> AsRef<Self> for Vector<N> {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<const N: usize> AsMut<[f32; N]> for Vector<N> {
    #[inline]
    fn as_mut(&mut self) -> &mut [f32; N] {
        self.inner_mut()
    }
}

impl<const N: usize> AsMut<[f32]> for Vector<N> {
    #[inline]
    fn as_mut(&mut self) -> &mut [f32] {
        self.inner_mut()
    }
}

impl<const N: usize> AsMut<Self> for Vector<N> {
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<I, const N: usize> Index<I> for Vector<N>
where
    [f32]: Index<I>,
{
    type Output = <[f32] as Index<I>>::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.inner().index(index)
    }
}

impl<const N: usize> IndexMut<usize> for Vector<N> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.inner_mut().index_mut(index)
    }
}

impl<const N: usize> IntoIterator for Vector<N> {
    type IntoIter = array::IntoIter<Self::Item, N>;
    type Item = f32;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}

impl<const N: usize> core::ops::Neg for Vector<N> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        self.flip()
    }
}

impl<const N: usize> Sum<Vector<N>> for Option<Vector<N>> {
    fn sum<I: Iterator<Item = Vector<N>>>(iter: I) -> Self {
        iter.reduce(Add::add)
    }
}

impl<const N: usize> Sum for Vector<N> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(Add::add).unwrap_or_default()
    }
}
