use std::mem;

use linear_algebra::{UnitVector, Vector};
use utils::builder;

use crate::types::VertexAttrType;

pub trait Vertex: Sized {
    const ELEMENT_COUNT: usize;

    fn stride() -> usize {
        mem::size_of::<Self>()
    }

    fn offsets() -> [usize; Self::ELEMENT_COUNT];
    // fn size_ofs() -> [usize; Self::ELEMENT_COUNT];
    fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT];

    fn from_incomplete_triangle(triangle: &[IncompleteVertex; 3]) -> [Self; 3];
}

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

    pub fn new(position: Vector<3>, texture: Vector<2>) -> Self {
        Self {
            position,
            texture,
            ..Default::default()
        }
    }
}
