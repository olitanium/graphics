use core::mem;

use geometry::NoTransform;
use graphics::linear_algebra::Vector;
use graphics::types::VertexAttrType;
use graphics::vertex::Vertex;
use utils::error_boilerplate;

use crate::modelling::ExtendedVertex;
use crate::modelling::extended_vertex::IncompleteVertex;

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

#[derive(Debug)]
pub enum FromIncompleteVertexError {
    NoTangent(Box<[IncompleteVertex<2>; 3]>),
}
error_boilerplate!(FromIncompleteVertexError);

impl ExtendedVertex for QuadVertex {
    type FromIncompleteVertexError = FromIncompleteVertexError;
    // TODO: 2D transforms
    type PreferredTransform<'a> = NoTransform;

    type const LEN: usize = 2;

    fn from_incomplete_triangle(
        triangle: [IncompleteVertex<2>; 3],
    ) -> Result<[Self; 3], Self::FromIncompleteVertexError> {
        triangle.clone().try_map(
            |IncompleteVertex {
                 position, texture, ..
             }| {
                Ok(Self {
                    position,
                    tex_coordinate: texture.ok_or_else(|| {
                        FromIncompleteVertexError::NoTangent(Box::new(triangle.clone()))
                    })?,
                })
            },
        )
    }

    fn position(&self) -> Vector<2> {
        self.position
    }
}
