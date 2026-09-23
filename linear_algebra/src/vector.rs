//! Module for defining the `Vector` type.

mod array_maths;

use core::cmp::Ordering;
use core::iter::Sum;
use core::marker::Destruct;
use core::ops::{Add, AddAssign, Div, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign};
use core::slice::SliceIndex;
use core::{array, iter, ptr};

use super::matrix::Matrix;
use crate::VectorPreferredFloat;
use crate::vector::array_maths::{div, mul};

mod unit_vector;
use float_eq::NearlyEq as _;
pub use unit_vector::UnitVector;

#[cfg(test)]
mod test;

mod compact_unit;
pub use compact_unit::CompactUnitVector;

/// Cartesian vector, comprised of the orthonormal x, y, z, ... (depending on N)
/// basis.
#[repr(transparent)]
#[derive(Copy, Debug)]
pub struct Vector<const N: usize, T = VectorPreferredFloat>(pub(crate) [T; N]);

const impl<const N: usize, T> Clone for Vector<N, T>
where
    [T; N]: [const] Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.0.clone())
    }
}

const impl<const N: usize, T> Default for Vector<N, T>
where
    T: const Default,
{
    // Would love to do as a #[derive()]
    #[inline]
    fn default() -> Self {
        Vector::new([const { T::default() }; N])
    }
}

const impl<const N: usize> PartialEq for Vector<N, VectorPreferredFloat> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        array_maths::nearly_eq(self.inner(), other.inner())
    }
}

impl<const N: usize> Eq for Vector<N, VectorPreferredFloat> {}

impl<const N: usize> PartialOrd for Vector<N, VectorPreferredFloat> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const N: usize> Ord for Vector<N, VectorPreferredFloat> {
    fn cmp(&self, other: &Self) -> Ordering {
        for (a, b) in iter::zip(self.iter(), other.iter()) {
            match a.total_cmp(b) {
                a @ (Ordering::Greater | Ordering::Less) => return a,
                Ordering::Equal => {}
            }
        }
        Ordering::Equal
    }
}

/// Convenience macro to write new vectors, rather than having to write
/// `Vector::new([...])`.
#[macro_export]
macro_rules! vector {
    ($($num:expr),+ $(,)?) => {
        $crate::Vector::new([$($num,)+])
    };
    ($num:expr; $len:expr) => {
        $crate::Vector::new([$num; $len])
    };
}

macro_rules! vector_elem {
    (0) => {
        pub const fn x(self) -> T
        where
            Self: [const] Destruct,
        {
            let [x, ..] = self.0;
            x
        }

        pub const fn x_mut(&mut self) -> &mut T {
            let [x, ..] = &mut self.0;
            x
        }
    };
    (1) => {
        pub const fn y(self) -> T
        where
            Self: [const] Destruct,
        {
            let [_, x, ..] = self.0;
            x
        }

        pub const fn y_mut(&mut self) -> &mut T {
            let [_, x, ..] = &mut self.0;
            x
        }
    };
    (2) => {
        pub const fn z(self) -> T
        where
            Self: [const] Destruct,
        {
            let [_, _, x, ..] = self.0;
            x
        }

        pub const fn z_mut(&mut self) -> &mut T {
            let [_, _, x, ..] = &mut self.0;
            x
        }
    };
    (3) => {
        pub const fn w(self) -> T
        where
            Self: [const] Destruct,
        {
            let [_, _, _, x, ..] = self.0;
            x
        }

        pub const fn w_mut(&mut self) -> &mut T {
            let [_, _, _, x, ..] = &mut self.0;
            x
        }
    };
}

impl<T> Vector<1, T> {
    vector_elem!(0);
}
impl<T> Vector<2, T> {
    vector_elem!(0);

    vector_elem!(1);
}
impl<T> Vector<3, T> {
    vector_elem!(0);

    vector_elem!(1);

    vector_elem!(2);
}
impl<T> Vector<4, T> {
    vector_elem!(0);

    vector_elem!(1);

    vector_elem!(2);

    vector_elem!(3);
}

impl<const N: usize> Vector<N, VectorPreferredFloat> {
    /// The first component `1.0`, others `0.0`.
    pub const X: Self = Self::make_single_vec(0);
    /// The second component `1.0`, others `0.0`.
    pub const Y: Self = Self::make_single_vec(1);
    /// The third component `1.0`, others `0.0`.
    pub const Z: Self = Self::make_single_vec(2);
    /// all components `0.0`.
    pub const ZERO: Self = Self::new([0.0; N]);

    #[inline]
    /// Helper function to create above constants.
    const fn make_single_vec(position: usize) -> Self {
        let mut vec = Self::ZERO;
        let Some(inner) = vec.inner_mut().get_mut(position) else {
            return vec;
        };
        *inner = 1.0;
        vec
    }

    /// Shortcut the normalisation process and assert that the given vector's
    /// magnitude is `1.0`. The vector must be have a magnitude of `1.0`, or
    /// this will break future optimisations.
    #[must_use]
    #[inline]
    pub const fn new_normal_unchecked(
        input: [VectorPreferredFloat; N],
    ) -> UnitVector<N, VectorPreferredFloat> {
        UnitVector::new_unchecked(input)
    }

    /// Check whether each of the vector's components are on or very close to
    /// zero.
    #[inline]
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        array_maths::is_zero(self.inner())
    }

    /// Find the length of the vector.
    #[inline]
    #[must_use]
    pub fn magnitude(self) -> VectorPreferredFloat {
        self.magnitude_sq().sqrt()
    }

    pub fn inverse_magnitude(&self) -> VectorPreferredFloat {
        1.0 / self.magnitude()
    }

    /// # Panics
    ///
    /// Panics when the vector is the Zero vector.
    #[must_use]
    #[inline]
    #[deprecated = "use the `try` method and handle the error"]
    pub fn normalize_or_panic(self) -> UnitVector<N, VectorPreferredFloat> {
        let Some(vec) = self.try_normalize() else {
            #[expect(clippy::panic, reason = "deprecated function, being phased out")]
            (panic!("tried normalizing a zero vector"))
        };
        vec
    }

    /// Try to normalize a vector, returning `None` if the vector is zero.
    #[must_use]
    #[inline]
    pub fn try_normalize(self) -> Option<UnitVector<N, VectorPreferredFloat>> {
        let one_len = 1.0 / self.magnitude();
        one_len
            .is_finite()
            .then(|| UnitVector::new_unchecked(self.0.map(|x| x * one_len)))
    }

    // pub const fn truncate_with<F, const M: usize>(self, f: F) -> Vector<M,
    // f32> where
    // F: [const] FnMut() -> f32,
    // {
    // let array_vec = ArrayVec::from_array(self.0);
    //
    // let new_arr = array_vec.into_array_with_new_len(f); Vector::new(new_arr)
    // }

    /// Change the size of the vector, filling in the gaps with `0.0`.
    #[must_use]
    #[inline]
    pub const fn truncate<const M: usize>(self) -> Vector<M, VectorPreferredFloat> {
        let array = array::from_fn(const |index| self.0.get(index).copied().unwrap_or(0.0));
        Vector::new(array)
    }

    /// Reflect the incoming ray `self` about a normal.
    #[must_use]
    #[inline]
    pub const fn reflect(self, normal: UnitVector<N, VectorPreferredFloat>) -> Self
    where
        Self: [const] Clone,
    {
        let normal = normal.v();
        let projection = normal.dot(self);
        self - normal.scale(2.0 * projection)
    }

    pub fn colinear(mut self, mut other: Self) -> bool {
        const SMALLEST: f32 = 0.001;

        // normalize all `-small` to `0.0`
        fn normalize_zero(val: &mut f32) {
            if *val < 0.0 && *val > -SMALLEST {
                *val = 0.0;
            }
        }

        self.iter_mut().for_each(normalize_zero);
        other.iter_mut().for_each(normalize_zero);

        let factors = self.elem_wise_div(other);
        factors
            .into_inner()
            .array_windows()
            .all(|[a, b]| a.nearly_eq(b, SMALLEST, SMALLEST) | b.is_nan() | a.is_nan())
    }
}

impl<const N: usize, T> Vector<N, T> {
    /// Create a new vector with a length given by the array input.
    #[must_use]
    #[inline]
    pub const fn new(input: [T; N]) -> Self {
        Self(input)
    }

    /// Destructure the vector and return the inner array, of length `N`.
    #[must_use]
    #[inline]
    pub const fn into_inner(self) -> [T; N]
    where
        Self: [const] Destruct,
    {
        let Self(out) = self;
        out
    }

    /// Get a reference to the inner array.
    #[must_use]
    #[inline]
    pub const fn inner(&self) -> &[T; N] {
        &self.0
    }

    /// Safely get a particular index into the vector, returning None if out of
    /// bounds.
    #[inline]
    #[must_use]
    pub const fn get<I>(&self, index: I) -> Option<&<I as SliceIndex<[T]>>::Output>
    where
        I: [const] SliceIndex<[T]>,
    {
        self.0.get(index)
    }

    /// Get a mutable reference to the inner array.
    #[must_use]
    #[inline]
    pub const fn inner_mut(&mut self) -> &mut [T; N] {
        &mut self.0
    }

    /// Get a mutable iterator to the inner array.
    #[inline]
    pub const fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.inner_mut().iter_mut()
    }

    /// Get an iterator to the inner array.
    pub const fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner().iter()
    }

    /// Perform an elementwise function to each in the vector.
    #[must_use]
    #[inline]
    pub const fn map<F>(self, op: F) -> Self
    where
        // from `array::map`
        T: [const] Destruct,
        F: [const] FnMut(T) -> T + [const] Destruct,
    {
        let array = self.into_inner().map(op);
        Self::new(array)
    }

    /// Find the square of the magnitude of the vector. This is more performant
    /// that using `Vector::magnitude` followed by squaring.
    #[must_use]
    #[inline]
    pub const fn magnitude_sq(self) -> T
    where
        T: [const] Add<Output = T> + [const] Mul<Output = T> + [const] Default + [const] Destruct,
        // fixme: Should only be clone, but [f32; 3] does not impl [const] Clone yet
        Self: Copy,
    {
        let temp = self;
        self.dot(temp)
    }

    /// Perform the scalar (inner) product of two vectors.
    /// There are several interpretations of the inner product:
    ///
    /// - The sum of the elementwise product of two vectors
    /// - The length of the projection of one vector onto the other, scaled by
    ///   the length of the other.
    /// - others...
    #[inline]
    #[must_use]
    pub const fn dot(self, rhs: Self) -> T
    where
        T: [const] Add<Output = T> + [const] Mul<Output = T> + [const] Default,
        Self: [const] Destruct,
        T: [const] Destruct,
    {
        array_maths::dot(self.into_inner(), rhs.into_inner()).unwrap_or_default()
    }

    /// Scale a vector by a given scalar.
    #[inline]
    #[must_use]
    pub const fn scale<U>(self, scalar: U) -> Self
    where
        T: [const] Mul<U, Output = T>,
        // TODO: remove when `Vector::into_inner` no longer needs it
        Self: [const] Destruct,
        U: [const] Clone + [const] Destruct,
    {
        let lhs = array::from_fn(const |_| scalar.clone());
        let arr = array_maths::mul(self.into_inner(), lhs);
        Self::new(arr)
    }

    /// Multiply each element of the vector with it's corresponding element in
    /// the other.
    #[inline]
    #[must_use]
    pub const fn elem_wise_mul(self, other: Self) -> Self
    where
        T: [const] Mul<Output = T>,
        T: [const] Destruct,
        T: [const] Clone,
    {
        Self::new(mul(self.into_inner(), other.into_inner()))
    }

    /// Divide each element of the vector with it's corresponding element in
    /// the other.
    #[inline]
    #[must_use]
    pub const fn elem_wise_div(self, other: Self) -> Self
    where
        T: [const] Div<Output = T>,
        T: [const] Destruct,
        T: [const] Clone,
    {
        Self::new(div(self.into_inner(), other.into_inner()))
    }
}

impl Vector<3, VectorPreferredFloat> {
    /// Perform the vector (outer) product. This method yields a vector
    /// perpendicular to both, with a magnitude scaled by how perpendular the
    /// vectors are.
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

    /// Returns the original vector with a trailing `1.0`, for use in
    /// homogeneous contexts.
    #[inline]
    #[must_use]
    pub const fn homogeneous(self) -> Vector<4, VectorPreferredFloat> {
        let [this0, this1, this2] = self.into_inner();
        Vector::new([this0, this1, this2, 1.0])
    }
}

/// convenience macro to impl `ops` traits.
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
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
    }
}

const impl<const N: usize, T> SubAssign for Vector<N, T>
where
    Self: [const] Sub<Output = Self> + [const] Clone + [const] Destruct,
{
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.clone() - rhs;
    }
}

const impl<const N: usize, T> Mul<T> for Vector<N, T>
where
    T: [const] Mul<Output = T> + [const] Clone + [const] Destruct,
{
    type Output = Self;

    #[inline]
    fn mul(self, rhs: T) -> Self::Output {
        self.scale(rhs)
    }
}

const impl<const N: usize, T> MulAssign<T> for Vector<N, T>
where
    Self: [const] Destruct + [const] Mul<T, Output = Self> + [const] Clone,
{
    #[inline]
    fn mul_assign(&mut self, rhs: T) {
        *self = self.clone() * rhs;
    }
}

const impl<const N: usize, T> Mul<Vector<N, T>> for f32
where
    Vector<N, T>: [const] Mul<f32>,
{
    type Output = <Vector<N, T> as Mul<f32>>::Output;

    #[inline]
    fn mul(self, rhs: Vector<N, T>) -> Self::Output {
        rhs * self
    }
}

const impl<const R: usize> From<Matrix<R, 1>> for Vector<R, VectorPreferredFloat> {
    #[inline]
    fn from(value: Matrix<R, 1>) -> Self {
        let [arr] = value.into_inner();
        Self::new(arr)
    }
}

const impl<const N: usize> From<[f32; N]> for Vector<N, f32> {
    #[inline]
    fn from(value: [f32; N]) -> Self {
        Self::new(value)
    }
}

const impl<const N: usize, T> From<Vector<N, T>> for [T; N]
where
    T: [const] Destruct,
{
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
    fn neg(mut self) -> Self::Output {
        let mut index = 0;
        while let Some(value) = self.0.get_mut(index) {
            let temp = -value.clone();
            *value = temp;

            index += 1;
        }

        self
    }
}

impl<const N: usize, T> Sum<Vector<N, T>> for Option<Vector<N, T>>
where
    Vector<N, T>: Add<Output = Vector<N, T>>,
{
    #[inline]
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Vector<N, T>>,
    {
        iter.reduce(Add::add)
    }
}

impl<const N: usize, T> Sum for Vector<N, T>
where
    Option<Self>: Sum<Self>,
    Self: Default,
{
    #[inline]
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        Option::<Self>::sum(iter).unwrap_or_default()
    }
}

impl<const N: usize, T> AsRef<Vector<N, T>> for [T; N] {
    #[inline]
    fn as_ref(&self) -> &Vector<N, T> {
        // SAFETY: reinterpret the reference as an array, this is similar to how
        // it's done in `array::from_ref`
        unsafe { &*ptr::from_ref(self).cast() }
    }
}
