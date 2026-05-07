use std::array;
use std::iter::Sum;
use std::ops::Index;

use crate::Vector;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnitVector<const N: usize>(pub(crate) Vector<N>);

impl<const N: usize> UnitVector<N> {
    const fn const_default() -> Self {
        let mut out = [0.0; N];
        if let Some(x) = out.get_mut(0) {
            *x = 1.0;
        }
        Self::new_unchecked(out)
    }
}

impl<const N: usize> Default for UnitVector<N> {
    fn default() -> Self {
        const { Self::const_default() }
    }
}

impl UnitVector<3> {
    pub const X: Self = Self::new_unchecked([1.0, 0.0, 0.0]);
    pub const Y: Self = Self::new_unchecked([0.0, 1.0, 0.0]);
    pub const Z: Self = Self::new_unchecked([0.0, 0.0, 1.0]);
}

impl<const N: usize> UnitVector<N> {
    #[inline(always)]
    pub const fn v(self) -> Vector<N> {
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

impl<const N: usize> Sum<UnitVector<N>> for Vector<N> {
    fn sum<I: Iterator<Item = UnitVector<N>>>(iter: I) -> Self {
        iter.map(|x| x.v()).sum()
    }
}
