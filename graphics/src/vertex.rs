use std::mem;

use linear_algebra::Vector;

use crate::types::VertexAttrType;

pub trait Vertex: Copy {
    const ELEMENT_COUNT: usize;

    fn stride() -> usize {
        mem::size_of::<Self>()
    }

    fn offsets() -> [usize; Self::ELEMENT_COUNT];

    fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT];
}

#[derive(Debug, Clone, Copy)]
pub struct NoVertex;

impl Vertex for NoVertex {
    const ELEMENT_COUNT: usize = 0;

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        []
    }

    fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT] {
        []
    }
}

impl Vertex for Vector<2> {
    const ELEMENT_COUNT: usize = 1;

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        [0]
    }

    fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT] {
        [(VertexAttrType::f32, 2)]
    }
}

impl Vertex for Vector<3> {
    const ELEMENT_COUNT: usize = 1;

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        [0]
    }

    fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT] {
        [(VertexAttrType::f32, 3)]
    }
}
