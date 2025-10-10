use graphics::linear_algebra::{UnitVector, Vector};
use utils::builder;

#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct IncompleteVertex<const N: usize> {
    pub position: Vector<N>,
    pub texture: Option<Vector<2>>,
    pub normal: Option<UnitVector<3>>,
    pub tangent: Option<UnitVector<3>>,
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
