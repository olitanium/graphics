use graphics::linear_algebra::{UnitVector, Vector};
use utils::builder;

use crate::TransformBy;
use crate::modelling::cubic::geometry::Pose;

#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct IncompleteVertex<const N: usize> {
    pub position: Vector<N>,
    pub texture: Option<Vector<2>>,
    pub normal: Option<UnitVector<3>>,
    pub tangent: Option<UnitVector<3>>,
}

impl TransformBy<Pose> for IncompleteVertex<3> {
    fn transform_by(&self, operator: &Pose) -> Self {
        Self {
            position: self.position.transform_by(operator),
            texture: self.texture,
            normal: self.normal.map(|unit| unit.transform_by(operator)),
            tangent: self.tangent.map(|unit| unit.transform_by(operator)),
        }
    }
}

impl TransformBy<Pose> for [IncompleteVertex<3>; 3] {
    fn transform_by(&self, operator: &Pose) -> Self {
        self.map(|ver| ver.transform_by(operator))
    }
}

impl<const N: usize> IncompleteVertex<N> {
    builder!(normal, opt_normal: Option<UnitVector<3>>);

    builder!(tangent, opt_tangent: Option<UnitVector<3>>);

    builder!(texture, opt_texture: Option<Vector<2>>);

    pub const fn new(position: Vector<N>) -> Self {
        Self {
            position,
            texture: None,
            normal: None,
            tangent: None,
        }
    }
}
