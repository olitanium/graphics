use std::mem;

use crate::types::VertexAttrType;

mod incomplete;
pub use incomplete::IncompleteVertex;
use linear_algebra::Vector;

pub trait Vertex: Copy {
    const ELEMENT_COUNT: usize;

    fn stride() -> usize {
        mem::size_of::<Self>()
    }

    fn offsets() -> [usize; Self::ELEMENT_COUNT];

    fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT];

    const DIMENSIONS: usize;

    fn from_incomplete_triangle(triangle: [IncompleteVertex; 3]) -> [Self; 3];
    fn position(&self) -> Vector<{ Self::DIMENSIONS }>;
}

impl Vertex for () {
    const DIMENSIONS: usize = 0;
    const ELEMENT_COUNT: usize = 0;

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        []
    }

    fn from_incomplete_triangle(_: [IncompleteVertex; 3]) -> [Self; 3] {
        [(), (), ()]
    }

    fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT] {
        []
    }

    fn position(&self) -> Vector<{ Self::DIMENSIONS }> {
        Vector::default()
    }
}

impl Vertex for Vector<2> {
    const DIMENSIONS: usize = 2;
    const ELEMENT_COUNT: usize = 1;

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        [0]
    }

    fn position(&self) -> Vector<{ Self::DIMENSIONS }> {
        *self
    }

    fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT] {
        [(VertexAttrType::f32, 2)]
    }

    fn from_incomplete_triangle(triangle: [IncompleteVertex; 3]) -> [Self; 3] {
        triangle.map(|vertex| vertex.position.truncate())
    }
}
