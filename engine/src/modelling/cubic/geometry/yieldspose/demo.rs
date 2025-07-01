use super::YieldsPose;
use graphics::linear_algebra::{UnitVector, Vector};
use crate::modelling::cubic::geometry::{Orientation, Pose};

#[derive(Debug, Copy, Clone)]
pub struct Demo;

impl YieldsPose for Demo {
    type Hint = f32;

    fn get_pose(&self, hint: Self::Hint) -> Pose {
        let hint = hint / 10.0;

        let position = [hint.sin(), hint.sin(), hint.cos()].into();
        let orientation = Orientation::new_from_to(
            position,
            Vector::from([0.0, 0.0, 0.0]),
            UnitVector::new_unchecked([0.0, 1.0, 0.0]),
        );

        Pose::new_from_orientation_translation(orientation, position)
    }
}
