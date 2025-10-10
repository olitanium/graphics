use graphics::linear_algebra::{UnitVector, Vector};

use crate::modelling::cubic::geometry::Pose;

pub trait TransformBy<By> {
    fn transform_by(&self, operator: &By) -> Self;
}

impl TransformBy<Pose> for Vector<3> {
    fn transform_by(&self, pose: &Pose) -> Self {
        let translation_as_pose = Pose::new_from_translation(*self);
        let current_pos = pose.transform_by(&translation_as_pose);
        current_pos.translation()
    }
}

impl TransformBy<Pose> for UnitVector<3> {
    fn transform_by(&self, pose: &Pose) -> Self {
        let orientation = pose.orientation();
        let vector = orientation.act_on(self.v());
        vector.normalize()
    }
}

impl TransformBy<Pose> for Pose {
    fn transform_by(&self, pose: &Pose) -> Self {
        let dual_quaternion = (pose.dual_quaternion.u() * self.dual_quaternion.u()).normalize();

        Self { dual_quaternion }
    }
}

pub trait Transform<T> {
    fn transform(&self, input: &T) -> T;
}

impl<U, T: TransformBy<U>> Transform<T> for U {
    fn transform(&self, input: &T) -> T {
        input.transform_by(self)
    }
}

pub struct NoTransform;

impl<T: Copy> TransformBy<NoTransform> for T {
    fn transform_by(&self, _: &NoTransform) -> Self {
        *self
    }
}
