mod array_maths;

use core::array;
use core::ops::{Index, IndexMut};
use std::iter::Sum;
use std::marker::Destruct;
use std::mem;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use std::slice::SliceIndex;

use super::matrix::Matrix;
use crate::vector::array_maths::mul;

mod unit_vector;
pub use unit_vector::UnitVector;

mod compact_unit;
pub use compact_unit::CompactUnitVector;

#[repr(transparent)]
#[derive(Copy, Debug)]
pub struct Vector<const N: usize, T = f32>(pub(crate) [T; N]);

const impl<const N: usize, T> Clone for Vector<N, T>
where
    [T; N]: [const] Clone,
{
    fn clone(&self) -> Self {
        Self::new(self.0.clone())
    }
}

const impl<const N: usize> Default for Vector<N, f32> {
    fn default() -> Self {
        Self::ZERO
    }
}

const impl<const N: usize> PartialEq for Vector<N, f32> {
    fn eq(&self, other: &Self) -> bool {
        array_maths::eq(self.into_inner(), other.into_inner())
    }
}

impl<const N: usize> Vector<N, f32> {
    pub const X: Self = Self::make_single_vec(0);
    pub const Y: Self = Self::make_single_vec(1);
    pub const Z: Self = Self::make_single_vec(2);
    pub const ZERO: Self = Self::new([0.0; N]);

    const fn make_single_vec(position: usize) -> Self {
        let mut vec = Self::ZERO;
        let Some(inner) = vec.inner_mut().get_mut(position) else {
            return vec;
        };
        *inner = 1.0;
        vec
    }

    /// # Safety
    ///
    /// The vector must be have a magnitude of 1.0, or this may break future
    /// optimisations
    #[must_use]
    #[inline]
    pub const fn new_normal_unchecked(input: [f32; N]) -> UnitVector<N, f32> {
        UnitVector::new_unchecked(input)
    }

    #[inline]
    #[must_use]
    pub const fn is_zero(self) -> bool {
        array_maths::is_zero(self.into_inner())
    }

    pub fn magnitude(self) -> f32 {
        self.magnitude_sq().sqrt()
    }

    /// # Panics
    #[must_use]
    #[inline]
    pub fn normalize(self) -> UnitVector<N, f32> {
        let Some(vec) = self.try_normalize() else {
            panic!("tried normalizing a zero vector");
        };
        vec
    }

    pub fn try_normalize(self) -> Option<UnitVector<N, f32>> {
        let one_len = 1.0 / self.magnitude();
        one_len
            .is_finite()
            .then(|| Self::new_normal_unchecked(self.0.map(|x| x * one_len)))
    }

    pub const fn truncate<const M: usize>(self) -> Vector<M, f32> {
        // CONST-HACK
        // let array = array::from_fn(|index|
        // self.0.get(index).copied().unwrap_or(0.0)); Vector::new(array)

        let mut array = [0.0; _];

        let mut index = 0;
        while let (Some(value), Some(dest)) = (self.0.get(index).copied(), array.get_mut(index)) {
            *dest = value;
            index += 1;
        }

        Vector::new(array)
    }

    pub const fn reflect(self, normal: UnitVector<N, f32>) -> Self
    where
        Self: [const] Clone,
    {
        let normal = normal.v();
        let projection = normal.dot(self);
        self - normal.scale(2.0 * projection)
    }
}

impl<const N: usize, T> Vector<N, T> {
    #[must_use]
    #[inline]
    pub const fn new(input: [T; N]) -> Self {
        Self(input)
    }

    #[must_use]
    #[inline]
    pub const fn into_inner(self) -> [T; N]
    where
        T: [const] Destruct, // TODO: shouldn't be needed because it is destructured
    {
        let Self(out) = self;
        out
    }

    #[must_use]
    #[inline]
    pub const fn inner(&self) -> &[T; N] {
        &self.0
    }

    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> usize {
        N
    }

    pub const fn get<I: [const] SliceIndex<[T]>>(
        &self,
        index: I,
    ) -> Option<&<I as SliceIndex<[T]>>::Output> {
        self.0.get(index)
    }

    #[must_use]
    #[inline]
    pub const fn inner_mut(&mut self) -> &mut [T; N] {
        &mut self.0
    }

    #[inline]
    pub const fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.inner_mut().iter_mut()
    }

    #[must_use]
    #[inline]
    pub fn map(self, op: impl Fn(T) -> T) -> Self {
        let array = self.into_inner().map(op);
        Self::new(array)
    }

    pub const fn magnitude_sq(self) -> T
    where
        T: [const] Clone
            + [const] Add<Output = T>
            + [const] Mul<Output = T>
            + [const] Default
            + Copy
            + [const] Destruct,
    {
        let temp = self;
        self.dot(temp)
    }

    #[inline]
    #[must_use]
    pub const fn dot(self, rhs: Self) -> T
    where
        T: [const] Add<Output = T> + [const] Mul<Output = T> + [const] Clone + [const] Default,
        T: [const] Destruct, // TODO: remove because `into_inner` is TODO remove
    {
        array_maths::dot(self.into_inner(), rhs.into_inner())
    }

    #[inline]
    #[must_use]
    pub const fn scale(mut self, scalar: T) -> Self
    where
        T: [const] Mul<Output = T> + [const] Clone + [const] Destruct,
    {
        // CONST-HACK
        // self.map(|elem| scalar * elem)

        let mut index = 0;

        while let Some(value) = self.0.get_mut(index) {
            *value = value.clone() * scalar.clone();

            index += 1;
        }

        self
    }

    pub const fn flip(mut self) -> Self
    where
        T: [const] Neg<Output = T>,
        T: [const] Clone,
        T: [const] Destruct,
    {
        let mut index = 0;
        // while index < N {
        // self.0[index] = -self.0[index];
        //
        // index += 1;
        // }

        while let Some(value) = self.0.get_mut(index) {
            let temp = -value.clone();
            *value = temp;

            index += 1;
        }

        self
    }

    #[must_use]
    pub const fn elem_wise_mul(self, other: Self) -> Self
    where
        T: [const] Mul<Output = T>,
        T: [const] Destruct,
        T: [const] Clone,
    {
        Vector::new(mul(self.into_inner(), other.into_inner()))
    }
}

impl Vector<3, f32> {
    #[inline]
    #[must_use]
    pub const fn cross(self, rhs: Self) -> Self {
        let [lhs0, lhs1, lhs2] = self.into_inner();
        let [rhs0, rhs1, rhs2] = rhs.into_inner();
        let out = [
            lhs1.mul_add(rhs2, -(lhs2 * rhs1)), // lhs1 * rhs2 - lhs2 * rhs1,
            lhs2.mul_add(rhs0, -(lhs0 * rhs2)), // lhs2 * rhs0 - lhs0 * rhs2,
            lhs0.mul_add(rhs1, -(lhs1 * rhs0)), // lhs0 * rhs1 - lhs1 * rhs0,
        ];
        Self::new(out)
    }

    #[must_use]
    pub const fn homogeneous(self) -> Vector<4, f32> {
        let [this0, this1, this2] = self.into_inner();
        Vector::new([this0, this1, this2, 1.0])
    }
}

macro_rules! impl_ops {
    ($trait:ident:: $fn:ident) => {
        use self::array_maths::$fn;
        const impl<const N: usize, T> core::ops::$trait for Vector<N, T>
        where
            T: [const] Clone + [const] Destruct + [const] core::ops::$trait<Output = T>,
        {
            type Output = Self;

            #[inline]
            fn $fn(self, rhs: Self) -> Self::Output {
                Self::new($fn(self.into_inner(), rhs.into_inner()))
            }
        }

        // impl<const N: usize> Vector<N> {
        // pub const fn $fn(self, rhs: Self) -> Self {
        // Self::new($crate::vector::array_maths::$fn(
        // self.into_inner(),
        // rhs.into_inner(),
        // ))
        // }
        // }

        // impl<const N: usize> core::ops::$trait<UnitVector<N>> for Vector<N> {
        // type Output = Self;
        //
        // #[inline]
        // fn $fn(self, rhs: UnitVector<N>) -> Self::Output {
        // Self::Output::new($fn(self.into_inner(), rhs.0.into_inner()))
        // }
        // }

        // impl<const N: usize> core::ops::$trait<Vector<N>> for UnitVector<N> {
        // type Output = Vector<N>;
        //
        // #[inline]
        // fn $fn(self, rhs: Vector<N>) -> Vector<N> {
        // Vector::new($fn(self.0.into_inner(), rhs.into_inner()))
        // }
        // }
    };
}

impl_ops!(Add::add);
impl_ops!(Sub::sub);

const impl<const N: usize, T> AddAssign for Vector<N, T>
where
    Self: [const] Add<Output = Self> + [const] Clone + [const] Destruct,
{
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
    }
}

const impl<const N: usize, T> SubAssign for Vector<N, T>
where
    Self: [const] Sub<Output = Self> + [const] Clone + [const] Destruct,
{
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.clone() - rhs;
    }
}

const impl<const N: usize, T> Mul<T> for Vector<N, T>
where
    T: [const] Mul<Output = T> + [const] Clone + [const] Destruct,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        self.scale(rhs)
    }
}

const impl<const N: usize, T> MulAssign<T> for Vector<N, T>
where
    Self: [const] Destruct + [const] Mul<T, Output = Self> + [const] Clone,
{
    fn mul_assign(&mut self, rhs: T) {
        *self = self.clone() * rhs;
    }
}

const impl<const N: usize> Mul<Vector<N, f32>> for f32 {
    type Output = Vector<N, f32>;

    fn mul(self, rhs: Vector<N, f32>) -> Self::Output {
        rhs * self
    }
}

impl<const R: usize> From<Matrix<R, 1>> for Vector<R, f32> {
    #[inline]
    fn from(value: Matrix<R, 1>) -> Self {
        let [arr] = value.into_inner();
        Vector::new(arr)
    }
}

impl<const N: usize> From<[f32; N]> for Vector<N, f32> {
    #[inline]
    fn from(value: [f32; N]) -> Self {
        Self::new(value)
    }
}

impl<const N: usize, T> From<Vector<N, T>> for [T; N] {
    #[inline]
    fn from(value: Vector<N, T>) -> Self {
        value.into_inner()
    }
}

impl<'a, const N: usize, T> TryFrom<&'a [T]> for Vector<N, T>
where
    T: Copy,
{
    type Error = <[T; N] as TryFrom<&'a [T]>>::Error;

    #[inline]
    fn try_from(value: &[T]) -> Result<Self, Self::Error> {
        value.try_into().map(Self::new)
    }
}

const impl<const N: usize, T> AsRef<[T; N]> for Vector<N, T> {
    #[inline]
    fn as_ref(&self) -> &[T; N] {
        self.inner()
    }
}

const impl<const N: usize, T> AsRef<[T]> for Vector<N, T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.inner()
    }
}

const impl<const N: usize, T> AsRef<Self> for Vector<N, T> {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

const impl<const N: usize, T> AsMut<[T; N]> for Vector<N, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T; N] {
        self.inner_mut()
    }
}

const impl<const N: usize, T> AsMut<[T]> for Vector<N, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.inner_mut()
    }
}

const impl<const N: usize, T> AsMut<Self> for Vector<N, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

const impl<I, const N: usize, T> Index<I> for Vector<N, T>
where
    [T]: [const] Index<I>,
{
    type Output = <[T] as Index<I>>::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.inner().index(index)
    }
}

const impl<const N: usize, T, U> IndexMut<U> for Vector<N, T>
where
    [T]: [const] IndexMut<U>,
{
    #[inline]
    fn index_mut(&mut self, index: U) -> &mut Self::Output {
        self.inner_mut().index_mut(index)
    }
}

impl<const N: usize, T> IntoIterator for Vector<N, T> {
    type IntoIter = array::IntoIter<Self::Item, N>;
    type Item = T;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}

const impl<const N: usize, T> Neg for Vector<N, T>
where
    T: [const] Neg<Output = T> + [const] Clone + [const] Destruct,
{
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        self.flip()
    }
}

impl<const N: usize, T> Sum<Vector<N, T>> for Option<Vector<N, T>>
where
    Vector<N, T>: Add<Output = Vector<N, T>>,
{
    fn sum<I: Iterator<Item = Vector<N, T>>>(iter: I) -> Self {
        iter.reduce(Add::add)
    }
}

impl<const N: usize, T> Sum for Vector<N, T>
where
    Option<Self>: Sum<Self>,
    Self: Default,
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Option::<Vector<N, T>>::sum(iter).unwrap_or_default()
    }
}

impl<const N: usize, T> AsRef<Vector<N, T>> for [T; N] {
    fn as_ref(&self) -> &Vector<N, T> {
        // transmute reference
        unsafe { mem::transmute(self) }
    }
}
