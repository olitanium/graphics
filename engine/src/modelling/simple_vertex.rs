use std::mem;

use graphics::linear_algebra::{UnitVector, Vector};
use graphics::types::VertexAttrType;
use graphics::vertex::{IncompleteVertex, Vertex};

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

    fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT] {
        use VertexAttrType as V;
        [(V::f32, 3), (V::f32, 2), (V::f32, 3), (V::f32, 3)]
    }

    fn from_incomplete_triangle(incomplete_triangle: &[IncompleteVertex; 3]) -> [Self; 3] {
        let new_normal = if incomplete_triangle.iter().all(|v| v.normal.is_none()) {
            let ab = incomplete_triangle[1].position - incomplete_triangle[0].position;
            let ac = incomplete_triangle[2].position - incomplete_triangle[0].position;
            ab.cross(ac).normalize()
        } else {
            UnitVector::new_unchecked([0.0, 0.0, 1.0])
        };

        let new_tangent = if incomplete_triangle.iter().all(|v| v.tangent.is_none()) {
            let ab = incomplete_triangle[1].position - incomplete_triangle[0].position;
            let ac = incomplete_triangle[2].position - incomplete_triangle[0].position;
            let z = incomplete_triangle[1].texture - incomplete_triangle[0].texture;
            let g = incomplete_triangle[2].texture - incomplete_triangle[0].texture;

            (ab.scale(g[1]) - ac.scale(z[1])) // .scale(1.0/(g[1]*z[0] - g[0]*z[1]))
                .normalize() // change of basis
        } else {
            UnitVector::new_unchecked([0.0, 0.0, 1.0])
        };

        incomplete_triangle.map(
            |IncompleteVertex {
                 position,
                 texture,
                 normal,
                 tangent,
                 ..
             }| SimpleVertex {
                position,
                texture,
                normal: normal.unwrap_or(new_normal),
                tangent: tangent.unwrap_or(new_tangent),
            },
        )
    }
}
