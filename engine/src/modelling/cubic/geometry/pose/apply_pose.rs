use graphics::linear_algebra::{UnitVector, Vector};

use crate::modelling::cubic::geometry::Pose;

pub trait ApplyPose {
    fn apply_pose(self, pose: Pose) -> Self;
}

impl ApplyPose for Vector<3> {
    fn apply_pose(self, pose: Pose) -> Self {
        let translation_as_pose = Pose::new_from_translation(self);
        let current_pos = translation_as_pose.apply_to(pose);
        current_pos.translation()
    }
}

impl ApplyPose for UnitVector<3> {
    fn apply_pose(self, pose: Pose) -> Self {
        let orientation = pose.orientation();
        let vector = orientation.act_on(self.v());
        vector.normalize()
    }
}

impl ApplyPose for Pose {
    fn apply_pose(self, pose: Pose) -> Self {
        let dual_quaternion = (pose.dual_quaternion.u() * self.dual_quaternion.u()).normalize();

        Self { dual_quaternion }
    }
}
