use graphics::linear_algebra::{UnitVector, Vector};

use crate::modelling::cubic::geometry::Pose;

pub trait TransformBy<By> {
    /// TransformBy takes an object, say a vector, and a transformation, say a
    /// matrix. The output is the result of applying the transformation to
    /// the object.
    ///
    /// In this sense, the operators are backwards in comparison to matrix
    /// multiplication, as vec.transform_by(matrix) is equal to matrix *
    /// vec.
    ///
    /// A seperate trait `Transform<Target>` which takes `Target` and returns
    /// `Target` exists, and there is a blanket impl for all types which
    /// implement TransformBy
    fn transform_by(&self, operator: &By) -> Self;
}

impl TransformBy<Pose> for Vector<3> {
    fn transform_by(&self, pose: &Pose) -> Self {
        let translation_as_pose = Pose::new_from_translation(*self);
        let current_pos = translation_as_pose.transform_by(pose);
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

/// Transform takes an input and returns the transformed input.
///
/// The operation order is reminiscent of matrix multiplication, in so far as
/// `matrix.transform(vec) == matrix * vec`
///
/// Do not implement this trait, instead implement transform_by, where private
/// members of the output type can be accessed. A blanket transform exists
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
