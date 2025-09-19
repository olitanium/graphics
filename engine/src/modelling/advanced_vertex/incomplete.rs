use graphics::linear_algebra::{UnitVector, Vector};
use utils::builder;

#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct IncompleteVertex {
    pub position: Vector<3>,
    pub texture: Vector<2>,
    pub normal: Option<UnitVector<3>>,
    pub tangent: Option<UnitVector<3>>,
}

impl IncompleteVertex {
    builder!(normal, opt_normal: Option<UnitVector<3>>);

    builder!(tangent, opt_tangent: Option<UnitVector<3>>);

    pub const fn new(position: Vector<3>, texture: Vector<2>) -> Self {
        Self {
            position,
            texture,
            normal: None,
            tangent: None,
        }
    }
}
