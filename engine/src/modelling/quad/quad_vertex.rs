use std::mem;

use graphics::linear_algebra::Vector;
use graphics::types::VertexAttrType;
use graphics::vertex::Vertex;

use crate::modelling::ExtendedVertex;
use crate::modelling::advanced_vertex::IncompleteVertex;

#[derive(Debug, Clone, Copy)]
pub struct QuadVertex {
    pub position: Vector<2>,
    pub tex_coordinate: Vector<2>,
}

impl Vertex for QuadVertex {
    type const ELEMENT_COUNT: usize = 2;

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        [
            mem::offset_of!(Self, position),
            mem::offset_of!(Self, tex_coordinate),
        ]
    }

    fn types_of() -> [VertexAttrType; Self::ELEMENT_COUNT] {
        let [vec2] = Vector::<2>::types_of();
        [vec2, vec2]
    }
}

impl ExtendedVertex<2> for QuadVertex {
    fn from_incomplete_triangle(triangle: [IncompleteVertex<2>; 3]) -> Option<[Self; 3]> {
        triangle.try_map(
            |IncompleteVertex {
                 position, texture, ..
             }| {
                Some(Self {
                    position,
                    tex_coordinate: texture?,
                })
            },
        )
    }

    fn position(&self) -> Vector<2> {
        self.position
    }
}
