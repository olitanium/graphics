mod array_maths;

use core::array;
use core::ops::{Index, IndexMut};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Mul, MulAssign, SubAssign};

use super::matrix::Matrix;

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Vector<const N: usize>([f32; N]);

impl<const N: usize> Default for Vector<N> {
    fn default() -> Self {
        Self::new_zero()
    }
}

impl<const N: usize> PartialEq for Vector<N> {
    fn eq(&self, other: &Self) -> bool {
        array_maths::is_zero((*self - *other).inner())
    }
}

impl<const N: usize> Vector<N> {
    #[must_use]
    #[inline]
    pub const fn new(input: [f32; N]) -> Self {
        Self(input)
    }

    #[must_use]
    #[inline]
    pub const fn new_zero() -> Self {
        Self::new([0.0; N])
    }

    /// # Safety
    ///
    /// The vector must be have a magnitude of 1.0, or this may break future
    /// optimisations
    #[must_use]
    #[inline]
    pub const unsafe fn new_normal_unchecked(input: [f32; N]) -> UnitVector<N> {
        UnitVector(Vector(input))
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

    pub const fn len(&self) -> usize {
        N
    }

    #[must_use]
    #[inline]
    pub const fn inner_mut(&mut self) -> &mut [f32; N] {
        &mut self.0
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut f32> {
        self.inner_mut().iter_mut()
    }

    #[must_use]
    #[inline]
    pub fn map(self, op: impl Fn(f32) -> f32) -> Self {
        let array = self.into_inner().map(op);
        Self::new(array)
    }

    pub fn magnitude_sq(self) -> f32 {
        self.dot(self)
    }

    pub fn magnitude(self) -> f32 {
        self.magnitude_sq().sqrt()
    }

    /// # Panics
    #[must_use]
    #[inline]
    pub fn normalize(self) -> UnitVector<N> {
        let one_len = 1.0 / self.magnitude();
        unsafe { Self::new_normal_unchecked(self.0.map(|x| x * one_len)) }
    }

    #[inline]
    #[must_use]
    pub fn is_zero(self) -> bool {
        array_maths::is_zero(self.inner())
    }

    #[inline]
    #[must_use]
    pub fn dot(self, rhs: Self) -> f32 {
        array_maths::dot(self.into_inner(), rhs.into_inner())
    }

    #[inline]
    #[must_use]
    pub fn from_to(from: Self, to: Self) -> Self {
        to - from
    }

    #[must_use]
    #[inline]
    pub fn distance_squared(self, to: Self) -> f32 {
        let direction = Self::from_to(self, to);
        direction.dot(direction)
    }

    #[inline]
    #[must_use]
    pub fn scale(self, scalar: f32) -> Self {
        self.map(|elem| scalar * elem)
    }

    #[inline]
    #[must_use]
    pub fn flip(mut self) -> Self {
        for x in self.inner_mut() {
            *x = -*x;
        }
        self
    }

    pub fn truncate<const M: usize>(self) -> Vector<M> {
        let array = array::from_fn(|index| self.0.get(index).copied().unwrap_or(0.0));
        Vector::new(array)
    }

    pub fn reflect(self, normal: UnitVector<N>) -> Self {
        let normal = normal.v();
        let projection = normal.dot(self);
        self - (2.0 * projection * normal)
    }
}

impl Vector<3> {
    #[inline]
    #[must_use]
    pub fn cross(self, rhs: Self) -> Self {
        let out = [
            self[1].mul_add(rhs[2], -(self[2] * rhs[1])), // self[1] * rhs[2] - self[2] * rhs[1],
            self[2].mul_add(rhs[0], -(self[0] * rhs[2])), // self[2] * rhs[0] - self[0] * rhs[2],
            self[0].mul_add(rhs[1], -(self[1] * rhs[0])), // self[0] * rhs[1] - self[1] * rhs[0],
        ];
        Self::new(out)
    }

    pub fn homogeneous(self) -> Vector<4> {
        Vector::new([self[0], self[1], self[2], 1.0])
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

impl<const N: usize> Index<usize> for Vector<N> {
    type Output = f32;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
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
        iter.reduce(Add::add).unwrap_or_else(Vector::new_zero)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct UnitVector<const N: usize>(Vector<N>);

impl<const N: usize> Default for UnitVector<N> {
    fn default() -> Self {
        let mut out = [0.0; N];
        if let Some(x) = out.get_mut(0) {
            *x = 1.0;
        }
        Self::new_unchecked(out)
    }
}

impl UnitVector<3> {
    pub const X: Self = Self::new_unchecked([1.0, 0.0, 0.0]);
    pub const Y: Self = Self::new_unchecked([0.0, 1.0, 0.0]);
    pub const Z: Self = Self::new_unchecked([0.0, 0.0, 1.0]);
}

impl<const N: usize> UnitVector<N> {
    #[inline(always)]
    pub fn v(self) -> Vector<N> {
        self.0
    }

    #[inline(always)]
    pub fn v_ref(&self) -> &Vector<N> {
        &self.0
    }

    pub const fn new_unchecked(input: [f32; N]) -> Self {
        Self(Vector(input))
    }
}

impl<const N: usize> core::ops::Neg for UnitVector<N> {
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        self.0 = -self.0;

        self
    }
}

impl<const N: usize> From<UnitVector<N>> for Vector<N> {
    #[inline]
    fn from(value: UnitVector<N>) -> Self {
        value.v()
    }
}

impl<const N: usize> From<UnitVector<N>> for [f32; N] {
    #[inline]
    fn from(value: UnitVector<N>) -> Self {
        value.v().into()
    }
}

impl<const N: usize> AsRef<Vector<N>> for UnitVector<N> {
    #[inline]
    fn as_ref(&self) -> &Vector<N> {
        &self.0
    }
}

impl<const N: usize> AsRef<[f32]> for UnitVector<N> {
    #[inline]
    fn as_ref(&self) -> &[f32] {
        self.0.inner()
    }
}

impl<const N: usize> Index<usize> for UnitVector<N> {
    type Output = f32;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.0.inner().index(index)
    }
}

impl<const N: usize> IntoIterator for UnitVector<N> {
    type IntoIter = array::IntoIter<Self::Item, N>;
    type Item = f32;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_inner().into_iter()
    }
}
