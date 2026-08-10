use array_vec::ArrayVec;
use geometry::{Pose, TransformBy};
use graphics::linear_algebra::{UnitVector, Vector};
use utils::builder;

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct IncompleteVertex<const N: usize> {
    pub position: Vector<N>,
    pub texture: Option<Vector<2>>,
    pub normal: Option<UnitVector<3>>,
    pub tangent: Option<UnitVector<3>>,
    pub weights: Option<ArrayVec<(usize, f32), 4>>,
}

impl TransformBy<Pose> for IncompleteVertex<3> {
    fn transform_by(&self, operator: &Pose) -> Self {
        Self {
            position: self.position.transform_by(operator),
            texture: self.texture,
            normal: self.normal.map(|unit| unit.transform_by(operator)),
            tangent: self.tangent.map(|unit| unit.transform_by(operator)),
            weights: self.weights.clone(),
        }
    }
}

// impl TransformBy<Pose> for [IncompleteVertex<3>; 3] {
// fn transform_by(&self, operator: &Pose) -> Self {
// self.clone().map(|ver| ver.transform_by(operator))
// }
// }

impl<const N: usize> IncompleteVertex<N> {
    builder!(normal, opt_normal: Option<UnitVector<3>>);

    builder!(tangent, opt_tangent: Option<UnitVector<3>>);

    builder!(texture, opt_texture: Option<Vector<2>>);

    builder!(weights, opt_weights: Option<ArrayVec<(usize, f32), 4>>);

    pub const fn new(position: Vector<N>) -> Self {
        Self {
            position,
            texture: None,
            normal: None,
            tangent: None,
            weights: None,
        }
    }
}
