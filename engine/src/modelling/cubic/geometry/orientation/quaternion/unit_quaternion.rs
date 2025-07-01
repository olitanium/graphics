use std::ops;

use super::Quaternion;
use graphics::linear_algebra::Matrix;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitQuaternion(Quaternion);

impl UnitQuaternion {
    pub fn normalize_quaternion(input: Quaternion) -> Self {
        let one_magnitude = 1.0 / input.magnitude();
        let quaternion = input.scale(one_magnitude);

        Self(quaternion)
    }

    pub fn q(self) -> Quaternion {
        self.0
    }

    pub fn inverse(self) -> Self {
        Self(self.0.conjugate())
    }

    // SAFETY: matrix must be a pure rotation matrix
    pub fn from_matrix(mat: Matrix<3, 3>) -> Self {
        // courtesy of http://marc-b-reynolds.github.io/quaternions/2017/08/08/QuatRotMatrix.html
        #[rustfmt::ignore]
        let [m00, m10, m20, m01, m11, m21, m02, m12, m22] = mat.col_major();

        let t1 = m22 + m11;
        let t0 = m00 + t1;

        if t0 > 0.0 {
            let real = 1.0 + t0;
            let i = m21 - m12;
            let j = m02 - m20;
            let k = m10 - m01;
            return Quaternion::new(real, i, j, k).normalize();
        }

        let t0 = m00 - t1;

        if t0 > 0.0 {
            let real = m21 - m12;
            let i = 1.0 + t0;
            let j = m01 + m10;
            let k = m02 + m20;

            return Quaternion::new(real, i, j, k).normalize();
        }

        let t0 = m11 - m22;
        let t1 = 1.0 - m00;

        if t0 > 0.0 {
            let real = m02 - m20;
            let i = m01 + m10;
            let j = t1 + t0;
            let k = m12 + m21;
            return Quaternion::new(real, i, j, k).normalize();
        }

        let real = m10 - m01;
        let i = m02 + m20;
        let j = m12 + m21;
        let k = t1 - t0;
        return Quaternion::new(real, i, j, k).normalize();
    }
}

impl Default for UnitQuaternion {
    fn default() -> Self {
        Self(Quaternion::identity())
    }
}

impl ops::Neg for UnitQuaternion {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let neg_inner = -self.0;
        Self(neg_inner)
    }
}
