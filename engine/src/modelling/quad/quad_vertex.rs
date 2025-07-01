use std::mem;

use graphics::buffers::IncompleteVertex;
use graphics::types::VertexAttrType;
use graphics::Vertex;
use graphics::linear_algebra::Vector;

#[derive(Debug, Clone, Copy)]
pub struct QuadVertex {
    pub position: Vector<2>,
    pub tex_coordinate: Vector<2>,
}

impl Vertex for QuadVertex {
    const ELEMENT_COUNT: usize = 2;

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        [
            mem::offset_of!(Self, position),
            mem::offset_of!(Self, tex_coordinate),
        ]
    }

    fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT] {
        use VertexAttrType as V;
        [
            (V::f32, 2),
            (V::f32, 2),
        ]
    }

    fn from_incomplete_triangle(triangle: &[graphics::buffers::IncompleteVertex; 3]) -> [Self; 3] {
        triangle.map(|IncompleteVertex {position, texture, ..}| {
            Self { position: position.truncate(), tex_coordinate: texture }
        } )
    }
}
