use geometry::{Orientation, Pose};
use graphics::linear_algebra::{UnitVector, Vector};

use super::CameraPose;

#[derive(Debug, Copy, Clone)]
pub struct Demo;

impl CameraPose for Demo {
    type Hint = f32;

    fn get_pose(&self, hint: &Self::Hint) -> Pose {
        let hint = hint / 10.0;

        let position = [hint.sin(), hint.sin(), hint.cos()].into();
        let orientation = Orientation::new_from_to(position, Vector::ZERO, UnitVector::Y)
            .expect("position never below `Vector::ZERO`");

        Pose::new_from_orientation_translation(orientation, position)
    }
}
