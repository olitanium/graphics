//! Module to define the Unit Vector, where the magnitude is equal to `1.0`.
use core::marker::Destruct;
use core::{array, ops};

use crate::{Vector, VectorPreferredFloat};

/// `UnitVector` with magnitude equal to `1.0`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct UnitVector<const N: usize, T = VectorPreferredFloat>(pub(crate) Vector<N, T>);

#[macro_export]
macro_rules! unit_vector {
    ($($(0.0)?),*) => {
        compile_error!("cannot have zero UnitVector literal")
    };

    ($($num:literal),+) => {
        // SAFETY: checked above for zero vector
        #[expect(clippy::macro_metavars_in_unsafe, reason = "this is always safe as checked above")]
        unsafe { $crate::Vector::new([$($num),*]).try_normalize().unwrap_unchecked() }
    }
}

impl<const N: usize, T> PartialEq for UnitVector<N, T>
where
    Vector<N, T>: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

const impl<const N: usize> Default for UnitVector<N, VectorPreferredFloat> {
    #[inline]
    fn default() -> Self {
        Self::X
    }
}

impl<const N: usize> UnitVector<N, VectorPreferredFloat> {
    /// `UnitVector([1.0, 0.0, 0.0, ...])` as long as asked for.
    pub const X: Self = Self(Vector::X);
}

impl UnitVector<2, VectorPreferredFloat> {
    /// `UnitVector([0.0, 1.0, 0.0, ...])` as long as asked for.
    pub const Y2: Self = Self(Vector::Y);
}

impl UnitVector<3, VectorPreferredFloat> {
    /// `UnitVector([0.0, 1.0, 0.0, ...])` as long as asked for.
    pub const Y: Self = Self(Vector::Y);
    /// `UnitVector([0.0, 0.0, 1.0, ...])` as long as asked for.
    pub const Z: Self = Self(Vector::Z);
}

impl<const N: usize, T> UnitVector<N, T> {
    /// Unwrap the `UnitVector` into it's inner `Vector`. Most vector operations
    /// happen with the `Vector` type.
    #[must_use]
    #[inline]
    pub const fn v(self) -> Vector<N, T>
    where
        Self: [const] Destruct,
    {
        let Self(out) = self;
        out
    }

    /// Get a reference to the inner `Vector`.
    #[inline]
    pub const fn v_ref(&self) -> &Vector<N, T> {
        &self.0
    }

    /// Create a new `UnitVector` without checking the magnitude.
    #[inline]
    pub const fn new_unchecked(input: [T; N]) -> Self {
        Self(Vector(input))
    }

    /// Skip the `Vector` and go straight to the inner array.
    #[inline]
    pub const fn into_inner(self) -> [T; N]
    where
        Vector<N, T>: [const] Destruct,
    {
        self.0.into_inner()
    }
}

impl<const N: usize, T> ops::Neg for UnitVector<N, T>
where
    Vector<N, T>: ops::Neg<Output = Vector<N, T>>,
{
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl<const N: usize, T> From<UnitVector<N, T>> for Vector<N, T> {
    #[inline]
    fn from(value: UnitVector<N, T>) -> Self {
        value.v()
    }
}

impl<const N: usize, T> From<UnitVector<N, T>> for [T; N] {
    #[inline]
    fn from(value: UnitVector<N, T>) -> Self {
        value.v().into_inner()
    }
}

impl<const N: usize, T> AsRef<Vector<N, T>> for UnitVector<N, T> {
    #[inline]
    fn as_ref(&self) -> &Vector<N, T> {
        &self.0
    }
}

impl<const N: usize, T> AsRef<[T]> for UnitVector<N, T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.0.inner()
    }
}

impl<const N: usize, T, I> ops::Index<I> for UnitVector<N, T>
where
    [T]: ops::Index<I>,
{
    type Output = <[T] as ops::Index<I>>::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.0.inner().index(index)
    }
}

impl<const N: usize, T> IntoIterator for UnitVector<N, T> {
    type IntoIter = array::IntoIter<Self::Item, N>;
    type Item = T;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_inner().into_iter()
    }
}
