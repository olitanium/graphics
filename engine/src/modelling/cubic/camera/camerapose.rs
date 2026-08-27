use geometry::{Orientation, Pose};
use graphics::linear_algebra::Vector;

pub trait CameraPose {
    type Hint;

    fn get_pose(&self, hint: &Self::Hint) -> Pose;

    fn get_position(&self, hint: &Self::Hint) -> Vector<3> {
        self.get_pose(hint).translation()
    }

    fn get_orientation(&self, hint: &Self::Hint) -> Orientation {
        self.get_pose(hint).orientation()
    }
}

impl CameraPose for Pose {
    type Hint = ();

    fn get_pose(&self, _: &Self::Hint) -> Pose {
        *self
    }
}
