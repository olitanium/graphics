use dual_number::{DualNumber, DualQuaternion, UnitDualQuaternion};
use graphics::linear_algebra::Matrix;
use quaternion::Quaternion;

mod posesclerp;
use graphics::linear_algebra::Vector;
pub use posesclerp::PoseSclerp;

use super::Orientation;
use crate::TransformBy;
use crate::modelling::cubic::camera::CameraPose;
use crate::transform::Transform;

#[derive(Debug, Clone, Copy)]
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

    #[deprecated(note = "use Pose::IDENTITY")]
    pub fn identity() -> Self {
        Self::new_from_orientation_translation(Orientation::IDENTITY, Vector::ZERO)
    }

    pub fn new_from_dual(dual_quaternion: UnitDualQuaternion) -> Self {
        Self { dual_quaternion }
    }

    pub fn new_from_orientation_translation(
        orientation: Orientation,
        translation: Vector<3>,
    ) -> Self {
        let real_part = orientation.into_quaternion().q();
        let dual_part = 0.5 * Quaternion::from_scalar_vec(0.0, translation) * real_part;

        Self {
            dual_quaternion: DualQuaternion::new(real_part, dual_part).normalize(),
        }
    }

    pub fn new_from_orientation(orientation: Orientation) -> Self {
        Self::new_from_orientation_translation(orientation, Vector::ZERO)
    }

    pub fn new_from_translation(translation: Vector<3>) -> Self {
        Self::new_from_orientation_translation(Orientation::IDENTITY, translation)
    }

    pub fn into_inner(self) -> UnitDualQuaternion {
        self.dual_quaternion
    }

    pub fn inverse(mut self) -> Self {
        self.dual_quaternion = self.dual_quaternion.inverse();
        self
    }

    pub fn translation(self) -> Vector<3> {
        let dual = self.dual_quaternion.u();
        let qd = dual.real;
        let qrstar = dual.dual.conjugate();
        let translation_quaternion = 2.0 * qd * qrstar;
        let minus_new = translation_quaternion.vector();
        // HACK: WHY IS THIS SO? https://cs.gmu.edu/~jmlien/teaching/cs451/uploads/Main/dual-quaternion.pdf
        // says that there need be no minus
        -minus_new
    }

    pub fn orientation(self) -> Orientation {
        let new = self.dual_quaternion.u().real.normalize();

        Orientation::from_quaternion(new)
    }

    pub fn as_matrix(self) -> Matrix<4, 4> {
        Matrix::transform_translate(self.translation()) * self.orientation().as_matrix()
    }

    pub fn push_change(&mut self, pose: Self) {
        //*self = pose.apply_to(*self);
        *self = pose.transform(self);
    }

    #[deprecated(note = "use Transform")]
    pub fn apply_to<V: TransformBy<Pose>>(self, value: V) -> V {
        self.transform(&value)
    }
}

impl CameraPose for Pose {
    type Hint = ();

    fn get_pose(&self, _: &Self::Hint) -> Pose {
        *self
    }
}

#[cfg(test)]
mod test {
    use graphics::linear_algebra::UnitVector;

    use super::*;

    #[test]
    fn test_matrix() {
        let first_pose = Pose::new_from_orientation_translation(
            Orientation::new_from_to(
                [0.0, 0.0, 0.0].into(),
                [0.0, 0.0, 1.0].into(),
                UnitVector::Y,
            ),
            Vector::new([1.0, 1.0, 1.0]),
        );

        let second_pose = Pose::new_from_orientation_translation(
            Orientation::new_from_to(
                [0.0, 0.0, 0.0].into(),
                [1.0, 0.0, 0.0].into(),
                UnitVector::Y,
            ),
            Vector::new([0.0, 0.5, 0.5]),
        );

        let new_pose = second_pose.transform(&first_pose);

        let composite_matrix = first_pose.as_matrix() * second_pose.as_matrix();
        let other_matrix = new_pose.as_matrix();

        assert_eq!(composite_matrix, other_matrix);
    }
}
