use std::mem;

use linear_algebra::{UnitVector, Vector};

use crate::buffers::vertex_buffer::Vertex;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SimpleVertex {
    // 0
    pub position: Vector<3>,
    // 1
    pub texture: Vector<2>,
    // 2
    pub normal: UnitVector<3>,
    // 3
    pub tangent: UnitVector<3>,
}

impl Vertex for SimpleVertex {
    const ELEMENT_COUNT: usize = 4;

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        [
            mem::offset_of!(Self, position),
            mem::offset_of!(Self, texture),
            mem::offset_of!(Self, normal),
            mem::offset_of!(Self, tangent),
        ]
    }

    fn types_of() -> [(crate::types::GLenum, usize); Self::ELEMENT_COUNT] {
        [
            (gl::FLOAT, 3),
            (gl::FLOAT, 2),
            (gl::FLOAT, 3),
            (gl::FLOAT, 3),
        ]
    }
}

#[cfg(test)]
mod test {}
