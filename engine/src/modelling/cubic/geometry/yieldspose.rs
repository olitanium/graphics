pub mod demo;

use super::pose::Pose;
use super::Orientation;
use crate::linear_algebra::Vector;

pub trait YieldsPose {
    type Hint;

    fn get_pose(&self, hint: Self::Hint) -> Pose;

    fn get_position(&self, hint: Self::Hint) -> Vector<3> {
        self.get_pose(hint).translation()
    }

    fn get_orientation(&self, hint: Self::Hint) -> Orientation {
        self.get_pose(hint).orientation()
    }
}
