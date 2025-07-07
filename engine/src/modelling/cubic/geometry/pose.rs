use dual_number::{DualQuaternion, UnitDualQuaternion};
use graphics::linear_algebra::Matrix;
use quaternion::Quaternion;

mod posesclerp;
use graphics::linear_algebra::Vector;
pub use posesclerp::PoseSclerp;

use super::{Orientation, YieldsPose};

#[derive(Debug, Clone, Copy)]
// #[non_exhaustive]
pub struct Pose {
    dual_quaternion: UnitDualQuaternion,
}

impl Default for Pose {
    fn default() -> Self {
        Self::new_from_orientation_translation(Default::default(), Default::default())
    }
}

impl Pose {
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

    pub fn into_inner(self) -> UnitDualQuaternion {
        self.dual_quaternion
    }

    pub fn inverse(mut self) -> Self {
        self.dual_quaternion = self.dual_quaternion.inverse();
        self
    }

    pub fn translation(self) -> Vector<3> {
        // let old = self.position;

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

    pub fn apply_after(self, pose: Self) -> Self {
        let dual_quaternion = (self.dual_quaternion.u() * pose.dual_quaternion.u()).normalize();

        Self { dual_quaternion }
    }
}

impl YieldsPose for Pose {
    type Hint = ();

    fn get_pose(&self, _: Self::Hint) -> Pose {
        *self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_matrix() {
        let first_pose = Pose::new_from_orientation_translation(
            Orientation::new_from_to(
                [0.0, 0.0, 0.0].into(),
                [0.0, 0.0, 1.0].into(),
                Vector::from([0.0, 1.0, 0.0]).normalize(),
            ),
            Vector::new([1.0, 1.0, 1.0]),
        );

        let second_pose = Pose::new_from_orientation_translation(
            Orientation::new_from_to(
                [0.0, 0.0, 0.0].into(),
                [1.0, 0.0, 0.0].into(),
                Vector::from([0.0, 1.0, 0.0]).normalize(),
            ),
            Vector::new([0.0, 0.5, 0.5]),
        );

        let new_pose = second_pose.apply_after(first_pose);

        let composite_matrix = second_pose.as_matrix() * first_pose.as_matrix();
        let other_matrix = new_pose.as_matrix();

        println!("composite: {composite_matrix:?}, {other_matrix:?}");

        assert_eq!(composite_matrix, other_matrix);
    }
}
