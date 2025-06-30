use std::mem;

use crate::buffers::vertex_buffer::Vertex;
use crate::linear_algebra::Vector;

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

    fn types_of() -> [(crate::types::GLenum, usize); Self::ELEMENT_COUNT] {
        [
            (gl::FLOAT, 2),
            (gl::FLOAT, 2),
        ]
    }
}
