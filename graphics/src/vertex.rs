use std::mem;

use crate::types::VertexAttrType;

mod incomplete;
pub use incomplete::IncompleteVertex;

pub trait Vertex: Sized {
    const ELEMENT_COUNT: usize;

    fn stride() -> usize {
        mem::size_of::<Self>()
    }

    fn offsets() -> [usize; Self::ELEMENT_COUNT];

    fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT];

    fn from_incomplete_triangle(triangle: &[IncompleteVertex; 3]) -> [Self; 3];
}

impl Vertex for () {
    const ELEMENT_COUNT: usize = 0;

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        []
    }

    fn from_incomplete_triangle(_: &[IncompleteVertex; 3]) -> [Self; 3] {
        [(), (), ()]
    }

    fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT] {
        []
    }
}
