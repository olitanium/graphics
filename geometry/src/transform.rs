use linear_algebra::{UnitVector, Vector};

use crate::{Orientation, Pose};

pub trait TransformBy<By>: Sized {
    /// `TransformBy` takes an object, say a vector, and a transformation, say a
    /// matrix. The output is the result of applying the transformation to
    /// the object.
    ///
    /// In this sense, the operators are backwards in comparison to matrix
    /// multiplication, as `vec.transform_by(matrix)` is equal to matrix *
    /// vec.
    ///
    /// Prefer implementing this trait over Transform.
    ///
    /// A seperate trait `Transform<Target>` which takes `Target` and returns
    /// `Target` exists, and there is a blanket impl for all types which
    /// implement `TransformBy`.
    #[must_use = "this function returns a new transform, rather than modifying the original \
                  in-place"]
    fn transform_by(self, operator: By) -> Self;
}

impl TransformBy<Pose> for Vector<3> {
    fn transform_by(self, operator: Pose) -> Self {
        let translation_as_pose = Pose::new_from_translation(self);
        let current_pos = translation_as_pose.transform_by(operator);
        current_pos.translation()
    }
}

impl TransformBy<Pose> for UnitVector<3> {
    fn transform_by(self, operator: Pose) -> Self {
        let orientation = operator.orientation();
        self.transform_by(orientation)
    }
}

impl TransformBy<Orientation> for Vector<3> {
    fn transform_by(self, operator: Orientation) -> Self {
        operator.act_on(self)
    }
}

impl TransformBy<Orientation> for UnitVector<3> {
    fn transform_by(self, operator: Orientation) -> Self {
        operator.act_on_unit(self)
    }
}

impl TransformBy<Self> for Pose {
    fn transform_by(self, pose: Self) -> Self {
        let dual_quaternion = (pose.dual_quaternion.u() * self.dual_quaternion.u()).normalize();

        Self { dual_quaternion }
    }
}

impl<T: TransformBy<U>, U> TransformBy<Option<U>> for T {
    fn transform_by(self, opt_operator: Option<U>) -> Self {
        if let Some(operator) = opt_operator {
            self.transform_by(operator)
        } else {
            self
        }
    }
}

impl<V: TransformBy<Pose>, const N: usize> TransformBy<Pose> for [V; N] {
    fn transform_by(self, operator: Pose) -> Self {
        self.map(|vertex| vertex.transform_by(operator))
    }
}

/// Transform takes an input and returns the transformed input.
///
/// The operation order is reminiscent of matrix multiplication, in so far as
/// `matrix.transform(vec) == matrix * vec`
///
/// Do not implement this trait, instead implement `transform_by`, where private
/// members of the output type can be accessed. A blanket transform exists
pub trait Transform<T> {
    fn transform(self, input: T) -> T;
}

impl<U, T: TransformBy<U>> Transform<T> for U {
    fn transform(self, input: T) -> T {
        input.transform_by(self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NoTransform;

impl<T> TransformBy<NoTransform> for T {
    fn transform_by(self, NoTransform: NoTransform) -> Self {
        self
    }
}
