use dual_number::{DualNumber, DualQuaternion, UnitDualQuaternion};
use linear_algebra::{Matrix, UnitVector, Vector};
use quaternion::Quaternion;

use crate::{Orientation, Transform as _, TransformBy as _};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    pub(crate) dual_quaternion: UnitDualQuaternion,
}

impl Default for Pose {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Pose {
    pub const IDENTITY: Self = Self {
        dual_quaternion: UnitDualQuaternion::new_normal_unchecked(DualNumber {
            real: Quaternion::IDENTITY,
            dual: Quaternion::ZERO,
        }),
    };

    #[must_use]
    pub const fn new_from_dual(dual_quaternion: UnitDualQuaternion) -> Self {
        Self { dual_quaternion }
    }

    pub const fn new_from_to_up(from: Vector<3>, to: Vector<3>, up: UnitVector<3>) -> Option<Self> {
        let forward = (to - from).try_normalize()?;
        Self::new_forward_up(from, forward, up)
    }

    pub const fn new_forward_up(
        from: Vector<3>,
        forward: UnitVector<3>,
        up: UnitVector<3>,
    ) -> Option<Self> {
        let orientation = Orientation::new_forward_up(forward, up)?;
        Some(Self::new_from_orientation_translation(orientation, from))
    }

    #[must_use]
    pub const fn new_from_orientation_translation(
        orientation: Orientation,
        translation: Vector<3>,
    ) -> Self {
        let real_part = orientation.into_quaternion().q();
        let dual_part = 0.5 * Quaternion::from_scalar_vec(0.0, translation) * real_part;

        Self {
            dual_quaternion: DualQuaternion::new(real_part, dual_part).normalize(),
        }
    }

    #[must_use]
    pub const fn new_from_orientation(orientation: Orientation) -> Self {
        Self::new_from_orientation_translation(orientation, Vector::ZERO)
    }

    #[must_use]
    pub const fn new_from_translation(translation: Vector<3>) -> Self {
        Self::new_from_orientation_translation(Orientation::IDENTITY, translation)
    }

    #[must_use]
    pub fn new_from_matrix(matrix: Matrix<4, 4>) -> Self {
        let [_, _, _, translation] = matrix.into_inner();
        let translation = Vector::new(translation).truncate();
        let orientation = matrix.truncate();

        Self::new_from_orientation_translation(Orientation::from_matrix(orientation), translation)
    }

    #[must_use]
    pub const fn into_inner(self) -> UnitDualQuaternion {
        self.dual_quaternion
    }

    #[must_use]
    pub const fn inverse(mut self) -> Self {
        self.dual_quaternion = self.dual_quaternion.inverse();
        self
    }

    #[must_use]
    pub const fn translation(self) -> Vector<3> {
        let dual = self.dual_quaternion.u();
        let qd = dual.real;
        let qrstar = dual.dual.conjugate();
        let translation_quaternion = 2.0 * qd * qrstar;
        let minus_new = translation_quaternion.vector();
        // HACK: WHY IS THIS SO? https://cs.gmu.edu/~jmlien/teaching/cs451/uploads/Main/dual-quaternion.pdf
        // says that there need be no minus
        -minus_new
    }

    #[must_use]
    pub const fn orientation(self) -> Orientation {
        let new = self.dual_quaternion.u().real.normalize();

        Orientation::from_unitquaternion(new)
    }

    #[must_use]
    pub fn as_transformation_matrix(self) -> Matrix<4, 4> {
        Matrix::transform_translate(self.translation()) * self.orientation().as_matrix()
    }

    #[must_use]
    pub const fn as_matrix_for_shader(self) -> Matrix<4, 2> {
        let DualNumber { real, dual } = self.into_inner().u();
        let [real0, real1, real2] = real.vector().into_inner();
        let [dual0, dual1, dual2] = dual.vector().into_inner();
        Matrix::from_col_major([
            [real0, real1, real2, real.scalar()],
            [dual0, dual1, dual2, dual.scalar()],
        ])
    }

    pub const fn push_change(&mut self, pose: Self) -> &mut Self {
        *self = (*self).transform_by(pose);
        self
    }

    pub const fn transform_x(self) -> UnitVector<3> {
        self.transform(UnitVector::X)
    }

    pub const fn transform_y(self) -> UnitVector<3> {
        self.transform(UnitVector::Y)
    }

    pub const fn transform_z(self) -> UnitVector<3> {
        self.transform(UnitVector::Z)
    }
}

#[cfg(test)]
mod test {
    use linear_algebra::{UnitVector, vector};

    use super::*;

    #[test]
    fn matrix() {
        let first_pose = Pose::new_from_orientation_translation(
            Orientation::new_from_to(Vector::ZERO, Vector::Z, UnitVector::Y).unwrap(),
            vector![1.0, 1.0, 1.0],
        );

        let second_pose = Pose::new_from_orientation_translation(
            Orientation::new_from_to(Vector::ZERO, Vector::X, UnitVector::Y).unwrap(),
            vector![0.0, 0.5, 0.5],
        );

        let new_pose = second_pose.transform(first_pose);

        let composite_matrix =
            second_pose.as_transformation_matrix() * first_pose.as_transformation_matrix();
        let other_matrix = new_pose.as_transformation_matrix();

        assert_eq!(composite_matrix, other_matrix);
    }
}
