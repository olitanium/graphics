use std::fmt::Debug;
use std::iter;
use std::ops::{self, Index, IndexMut};

use multi_zip::MultiZip;

use crate::vector::Vector;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq)]
pub struct Matrix<const R: usize, const C: usize>([Vector<R>; C]); // Col-Major!!!!!

impl<const R: usize, const C: usize> Default for Matrix<R, C> {
    #[inline]
    fn default() -> Self {
        Self::identity()
    }
}

impl<const R: usize, const C: usize> Debug for Matrix<R, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.row_iter() {
            let array = row.into_inner();
            writeln!(f, "{array:?}")?
        }
        Ok(())
    }
}

impl<const R: usize, const C: usize> Matrix<R, C> {
    #[inline]
    pub fn from_col_major(input: [impl Into<Vector<R>>; C]) -> Self {
        let vec_arr = input.map(Into::into);
        Self(vec_arr)
    }

    #[inline]
    pub fn from_row_major(input: [impl Into<Vector<C>>; R]) -> Self {
        Matrix::from_col_major(input).transpose()
    }

    /// # Panics
    /// Panics if the length of the slice is not as given by the compile-time
    /// constants
    #[inline]
    pub fn new_from_slice(input: &[Vector<R>]) -> Self {
        Self(input.try_into().unwrap())
    }

    #[inline]
    pub fn zeros() -> Self {
        Self::from_col_major([[0.0; R]; C])
    }

    #[inline]
    pub const fn into_inner(self) -> [Vector<R>; C] {
        self.0
    }

    #[inline]
    pub const fn inner(&self) -> &[Vector<R>; C] {
        &self.0
    }

    #[inline]
    pub fn inner_mut(&mut self) -> &mut [Vector<R>; C] {
        &mut self.0
    }

    #[inline]
    pub fn col_iter(&self) -> impl Iterator<Item = Vector<R>> + '_ {
        self.inner().iter().copied()
    }

    #[inline]
    pub fn col_iter_mut(&mut self) -> impl Iterator<Item = &mut Vector<R>> {
        self.inner_mut().iter_mut()
    }

    #[expect(clippy::missing_panics_doc)]
    #[inline]
    pub fn col_major(&self) -> [f32; C * R] {
        let x = self
            .col_iter()
            .flat_map(IntoIterator::into_iter)
            .collect::<Box<_>>();
        x.as_ref()
            .try_into()
            .expect("Number of entries is permitted by compile time constants")
    }

    pub fn col(&self, index: usize) -> Vector<R> {
        self.0[index]
    }

    #[expect(clippy::missing_panics_doc)]
    #[inline]
    pub fn row_iter(&self) -> impl Iterator<Item = Vector<C>> {
        let x = self.inner().map(Vector::into_iter);
        MultiZip::new(x).map(|vec| {
            Vector::try_from(vec.as_ref())
                .expect("Number of entries is permitted by compile time constants")
        })
    }

    #[expect(clippy::missing_panics_doc)]
    #[inline]
    pub fn row_major(&self) -> [f32; C * R] {
        let x = self
            .row_iter()
            .flat_map(IntoIterator::into_iter)
            .collect::<Box<_>>();
        x.as_ref()
            .try_into()
            .expect("Number of entries does not change when transposing")
    }

    #[inline]
    pub fn identity() -> Self {
        let mut output = Self::zeros();
        for i in 0..[R, C][usize::from(R < C)] {
            output[(i, i)] = 1.0;
        }
        output
    }

    #[inline]
    pub fn transpose(self) -> Matrix<C, R> {
        let matrix_as_vec = self.row_iter().collect::<Box<_>>();

        Matrix::new_from_slice(&matrix_as_vec)
    }

    #[inline]
    pub fn truncate<const R2: usize, const C2: usize>(self) -> Matrix<R2, C2> {
        let x = self
            .into_inner()
            .map(Vector::truncate)
            .into_iter()
            .chain(iter::repeat(Vector::ZERO))
            .take(C2)
            .collect::<Box<_>>();
        Matrix::new_from_slice(x.as_ref())
    }

    #[must_use]
    #[inline]
    pub fn map(mut self, op: impl Fn(f32) -> f32) -> Self {
        let x = self.inner_mut().map(|vec| vec.map(&op));
        Self::from_col_major(x)
    }
}

impl<const R: usize, const S: usize, const C: usize> ops::Mul<Matrix<S, C>> for Matrix<R, S> {
    type Output = Matrix<R, C>;

    #[inline]
    fn mul(self, rhs: Matrix<S, C>) -> Self::Output {
        Self::Output::new_from_slice(
            &rhs.col_iter()
                .map(|b_col| {
                    Vector::<R>::try_from(
                        self.row_iter()
                            .map(|val| val.dot(b_col))
                            .collect::<Box<_>>()
                            .as_ref(),
                    )
                    .unwrap()
                })
                .collect::<Box<_>>(),
        )
    }
}

impl<const R: usize> ops::MulAssign for Matrix<R, R> {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

macro_rules! impl_ops {
    ($trait:ident:: $fn:ident) => {
        impl<const R: usize, const C: usize> core::ops::$trait for Matrix<R, C> {
            type Output = Self;

            #[inline]
            fn $fn(self, rhs: Self) -> Self::Output {
                Self::Output::new_from_slice(
                    &self
                        .col_iter()
                        .zip(rhs.col_iter())
                        .map(|(a, b)| Vector::$fn(a, b))
                        .collect::<Box<_>>(),
                )
            }
        }
    };
}

impl_ops!(Add::add);
impl_ops!(Sub::sub);

impl<const R: usize, const C: usize> Index<(usize, usize)> for Matrix<R, C> {
    type Output = <Vector<R> as Index<usize>>::Output;

    #[inline]
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (row, col) = index;

        self.inner().index(col).index(row)
    }
}

impl<const R: usize, const C: usize> IndexMut<(usize, usize)> for Matrix<R, C> {
    #[inline]
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (row, col) = index;

        self.inner_mut().index_mut(col).index_mut(row)
    }
}

impl<const R: usize> From<Vector<R>> for Matrix<R, 1> {
    #[inline]
    fn from(value: Vector<R>) -> Self {
        Self::from_col_major([value])
    }
}

impl Matrix<4, 4> {
    pub fn transform_scale(sx: f32, sy: f32, sz: f32) -> Self {
        let mut matrix = Self::zeros();
        matrix[(0, 0)] = sx;
        matrix[(1, 1)] = sy;
        matrix[(2, 2)] = sz;
        matrix[(3, 3)] = 1.0;
        matrix
    }

    pub fn transform_translate(vec: Vector<3>) -> Self {
        let mut matrix = Self::identity();
        matrix[(0, 3)] = vec[0];
        matrix[(1, 3)] = vec[1];
        matrix[(2, 3)] = vec[2];
        matrix
    }

    pub fn transform_perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        let mut matrix = Self::zeros();
        let fovbytwo = fov * 0.5;
        matrix[(0, 0)] = 1.0 / (fovbytwo.tan() * aspect);
        matrix[(1, 1)] = 1.0 / fovbytwo.tan();
        matrix[(2, 2)] = -(far + near) / (far - near);
        matrix[(3, 2)] = -1.0;
        matrix[(2, 3)] = -2.0 * far * near / (far - near);
        matrix
    }

    pub fn transform_orthogonal(width: f32, height: f32, near: f32, far: f32) -> Self {
        Self::from_col_major([
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, 2.0 / height, 0.0, 0.0],
            [0.0, 0.0, -2.0 / (far - near), 0.0],
            [0.0, 0.0, -(far + near) / (far - near), 1.0],
        ])
    }
}
