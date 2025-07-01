use std::mem;

use linear_algebra::{UnitVector, Vector};

use crate::{buffers::vertex_array::IncompleteVertex, types::VertexAttrType};

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