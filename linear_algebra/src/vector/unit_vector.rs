use std::array;
use std::marker::Destruct;
use std::ops::{Index, Neg};

use crate::Vector;

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct UnitVector<const N: usize, T = f32>(pub(crate) Vector<N, T>);

impl<const N: usize, T> PartialEq for UnitVector<N, T>
where
    Vector<N, T>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<const N: usize> UnitVector<N, f32> {
    const fn const_default() -> Self {
        let mut out = [0.0; N];
        if let Some(x) = out.get_mut(0) {
            *x = 1.0;
        }
        Self::new_unchecked(out)
    }
}

const impl<const N: usize> Default for UnitVector<N, f32> {
    fn default() -> Self {
        const { Self::const_default() }
    }
}

impl UnitVector<3, f32> {
    pub const X: Self = Self::new_unchecked([1.0, 0.0, 0.0]);
    pub const Y: Self = Self::new_unchecked([0.0, 1.0, 0.0]);
    pub const Z: Self = Self::new_unchecked([0.0, 0.0, 1.0]);
}

impl<const N: usize, T> UnitVector<N, T> {
    #[inline(always)]
    pub const fn v(self) -> Vector<N, T>
    where
        Self: [const] Destruct,
    {
        let Self(out) = self;
        out
    }

    #[inline(always)]
    pub fn v_ref(&self) -> &Vector<N, T> {
        &self.0
    }

    pub const fn new_unchecked(input: [T; N]) -> Self {
        Self(Vector(input))
    }

    pub const fn into_inner(self) -> [T; N]
    where
        T: [const] Destruct,
    {
        self.0.into_inner()
    }
}

impl<const N: usize, T> core::ops::Neg for UnitVector<N, T>
where
    Vector<N, T>: Neg<Output = Vector<N, T>>,
{
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        self.0 = -self.0;

        self
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
        value.v().into()
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

impl<const N: usize, T, I> Index<I> for UnitVector<N, T>
where
    [T]: Index<I>,
{
    type Output = <[T] as Index<I>>::Output;

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

// impl<const N: usize, T> Sum<UnitVector<N>> for Vector<N> {
// fn sum<I: Iterator<Item = UnitVector<N>>>(iter: I) -> Self {
// iter.map(|x| x.v()).sum()
// }
// }
