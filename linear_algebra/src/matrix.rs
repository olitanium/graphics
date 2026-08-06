//! module for defining and storing matrix

use core::fmt::Debug;
use core::iter;
use core::ops::{self, Index, IndexMut};

use array_vec::ArrayVec;
use multi_zip::MultiZip;

use crate::vector::Vector;

#[cfg(test)]
mod test;

/// Matrix object in column major.
///
/// Features constructors to create new from row or column major, but column
/// major constructor is a no-op.
///
/// Comparable with other Matrices, this type offers matrix-like operations such
/// as multiplication and element-wise operataions.
///
/// # Example
///
/// ```rust
/// use linear_algebra::Matrix;
/// let one = Matrix::from_col_major([[1.0, 2.0], [3.0, 4.0]]);
/// let half = Matrix::from_row_major([[0.5, 1.0], [1.5, 2.0]]);
///
/// let two_half = 2.0 * half;
///
/// assert_eq!(one, two_half);
/// ```
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Matrix<const R: usize, const C: usize>([[f32; R]; C]); // Col-Major!!!!!

impl<const R: usize, const C: usize> PartialEq for Matrix<R, C> {
    fn eq(&self, other: &Self) -> bool {
        iter::zip(self.0, other.0).all(|(arr1, arr2)| {
            let ref_vec: &Vector<R> = arr1.as_ref();
            let ref_vec2: &Vector<R> = arr2.as_ref();
            ref_vec == ref_vec2
        })
    }
}

impl<const R: usize, const C: usize> Default for Matrix<R, C> {
    #[inline]
    fn default() -> Self {
        Self::identity()
    }
}

impl<const R: usize, const C: usize> Debug for Matrix<R, C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for row in self.row_iter() {
            writeln!(f, "{row:?}")?;
        }
        Ok(())
    }
}

impl<const R: usize, const C: usize> Matrix<R, C> {
    #[inline]
    #[must_use]
    pub const fn from_col_major(input: [[f32; R]; C]) -> Self {
        Self(input)
    }

    #[inline]
    #[must_use]
    pub fn from_row_major(input: [[f32; C]; R]) -> Self {
        Matrix::from_col_major(input).transpose()
    }

    /// # Panics
    /// Panics if the length of the slice is not as given by the compile-time
    /// constants
    #[inline]
    #[must_use]
    pub fn new_from_slice(input: &[[f32; R]]) -> Option<Self> {
        Some(Self(input.try_into().ok()?))
    }

    #[inline]
    #[must_use]
    pub const fn zeros() -> Self {
        Self::from_col_major([[0.0; R]; C])
    }

    #[inline]
    #[must_use]
    pub const fn into_inner(self) -> [[f32; R]; C] {
        self.0
    }

    #[inline]
    #[must_use]
    pub const fn inner(&self) -> &[[f32; R]; C] {
        &self.0
    }

    #[inline]
    pub const fn inner_mut(&mut self) -> &mut [[f32; R]; C] {
        &mut self.0
    }

    #[must_use]
    pub const fn col_flattened(&self) -> &[f32] {
        self.0.as_flattened()
    }

    pub const fn col_flattened_mut(&mut self) -> &mut [f32] {
        self.0.as_flattened_mut()
    }

    #[inline]
    pub fn col_iter(&self) -> impl Iterator<Item = [f32; R]> + '_ {
        self.inner().iter().copied()
    }

    #[inline]
    pub fn col_iter_mut(&mut self) -> impl Iterator<Item = &mut [f32; R]> {
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

    pub const fn col(&self, index: usize) -> Option<Vector<R, f32>> {
        self.0.get(index).copied().map(Vector::new)
    }

    pub const fn col_mut(&mut self, index: usize) -> Option<&mut [f32; R]> {
        self.0.get_mut(index)
    }

    #[must_use]
    pub const fn get(&self, (row, col): (usize, usize)) -> Option<f32> {
        let vec = self.col(col)?;
        vec.get(row).copied()
    }

    pub const fn get_mut(&mut self, (row, col): (usize, usize)) -> Option<&mut f32> {
        let vec = self.col_mut(col)?;
        vec.get_mut(row)
    }

    #[inline]
    pub fn row_iter(&self) -> impl Iterator<Item = [f32; C]> {
        let x = self.into_inner();
        MultiZip::new(x)
    }

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
    #[must_use]
    pub const fn identity() -> Self {
        let mut output = Self::zeros();

        let mut i = 0;

        while let Some(val) = output.get_mut((i, i)) {
            *val = 1.0;
            i += 1;
        }

        output
    }

    #[inline]
    #[must_use]
    pub fn transpose(self) -> Matrix<C, R> {
        let matrix_as_vec = self.row_iter().collect::<ArrayVec<[f32; C], R>>();
        let array = matrix_as_vec
            .try_into_array()
            .expect("RC matrix has as many elements as CR");
        Matrix::from_col_major(array)
    }

    #[inline]
    #[must_use]
    pub fn truncate<const R2: usize, const C2: usize>(self) -> Matrix<R2, C2> {
        let x = self
            .into_inner()
            .into_iter()
            .map(|arr| Vector::new(arr).truncate().into_inner())
            .chain(iter::repeat([0.0; _]))
            .collect::<ArrayVec<_, C2>>();

        let array = x
            .try_into_array()
            .expect("ArrayVec filling from unlimited iterator");

        Matrix::from_col_major(array)
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
        let self_transpose = self.transpose();
        Self::Output::from_col_major(rhs.into_inner().map(|b_col| {
            self_transpose
                .into_inner()
                .map(|val| Vector::new(val).dot(Vector::new(b_col)))
        }))
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
                Self::Output::from_col_major(
                    // TODO: would be great with array::zip_map
                    self.col_iter()
                        .zip(rhs.col_iter())
                        .map(|(a, b)| Vector::$fn(Vector::new(a), Vector::new(b)).into_inner())
                        .collect::<ArrayVec<_, C>>()
                        .try_into_array()
                        .expect("enough elements from input"),
                )
            }
        }
    };
}

impl_ops!(Add::add);
impl_ops!(Sub::sub);

impl<const R: usize, const C: usize> Index<(usize, usize)> for Matrix<R, C> {
    type Output = f32;

    /// # Panics
    /// Will panic is the index is put of bounds.
    #[inline]
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (row, col) = index;

        self.inner().index(col).index(row)
    }
}

impl<const R: usize, const C: usize> IndexMut<(usize, usize)> for Matrix<R, C> {
    /// # Panics
    /// Will panic is the index is put of bounds.
    #[inline]
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (row, col) = index;

        self.inner_mut().index_mut(col).index_mut(row)
    }
}

impl<const R: usize> From<Vector<R, f32>> for Matrix<R, 1> {
    #[inline]
    fn from(value: Vector<R, f32>) -> Self {
        Self::from_col_major([value.into_inner()])
    }
}

impl Matrix<2, 2> {
    /// Calculate the determinant of the matrix.
    #[must_use]
    #[inline]
    pub const fn determinant(&self) -> f32 {
        let [[a, c], [b, d]] = self.0;

        a * d - b * c
    }
}

impl Matrix<3, 3> {
    /// Calculate the determinant of the matrix.
    #[inline]
    #[must_use]
    pub const fn determinant(&self) -> f32 {
        let [[a, d, g], [b, e, h], [c, f, i]] = self.into_inner();

        a * e * i + b * f * g + c * d * h - c * e * g - b * d * i - a * f * h
    }
}

impl Matrix<4, 4> {
    /// Create a matrix to scale a 3 dimensional homogeneous vector (i.e. a
    /// `Vector<4>` where `x[3] == 1.0`).
    #[must_use]
    #[inline]
    pub const fn transform_scale(sx: f32, sy: f32, sz: f32) -> Self {
        Self::from_col_major([
            [sx, 0.0, 0.0, 0.0],
            [0.0, sy, 0.0, 0.0],
            [0.0, 0.0, sz, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Create a matrix to translate a 3 dimensional homogeneous vector (i.e. a
    /// `Vector<4>` where `x[3] == 1.0`).
    #[inline]
    #[must_use]
    pub const fn transform_translate(vec: Vector<3, f32>) -> Self {
        // let mut matrix = Self::identity();
        // let &mut [_, _, _, [ref mut x, ref mut y, ref mut z, _]] =
        // matrix.inner_mut(); let [vecx, vecy, vecz] = vec.into_inner();
        // x = vecx;
        // y = vecy;
        // z = vecz;
        //
        // matrix

        let [x, y, z] = vec.into_inner();
        Self::from_col_major([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [x, y, z, 1.0],
        ])
    }

    /// Create a matrix to transform a 3 dimensional homogeneous vector (i.e. a
    /// `Vector<4>` where `x[3] == 1.0`) into a perspective cube, with maximums
    /// at +/- 1.0 on x[0], x[1] and x[2], and the
    #[inline]
    #[must_use]
    pub fn transform_perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        let tan_fov = (fov * 0.5).tan();
        Self::from_col_major([
            [1.0 / (tan_fov * aspect), 0.0, 0.0, 0.0],
            [0.0, 1.0 / tan_fov, 0.0, 0.0],
            [0.0, 0.0, -(far + near) / (far - near), -1.0],
            [0.0, 0.0, -2.0 * far * near / (far - near), 0.0],
        ])
    }

    #[must_use]
    pub const fn transform_orthogonal(width: f32, height: f32, near: f32, far: f32) -> Self {
        Self::from_col_major([
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, 2.0 / height, 0.0, 0.0],
            [0.0, 0.0, -2.0 / (far - near), 0.0],
            [0.0, 0.0, -(far + near) / (far - near), 1.0],
        ])
    }
}
