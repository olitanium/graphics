use std::cell::LazyCell;
use std::mem;

use graphics::linear_algebra::{UnitVector, Vector};
use graphics::types::VertexAttrType;
use graphics::vertex::Vertex;

use crate::TransformBy;
use crate::modelling::advanced_vertex::{ExtendedVertex, IncompleteVertex};
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{CubicVertex, VertexHasTangent};
use crate::transform::Transform;

#[derive(Debug, Clone, Copy, PartialEq)]
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
    type const ELEMENT_COUNT: usize = 4;

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        [
            mem::offset_of!(Self, position),
            mem::offset_of!(Self, texture),
            mem::offset_of!(Self, normal),
            mem::offset_of!(Self, tangent),
        ]
    }

    fn types_of() -> [VertexAttrType; Self::ELEMENT_COUNT] {
        let [vec3] = Vector::<3>::types_of();
        let [vec2] = Vector::<2>::types_of();
        let [unitvec] = UnitVector::<3>::types_of();
        [vec3, vec2, unitvec, unitvec]
    }
}

impl CubicVertex for SimpleVertex {}

impl VertexHasTangent for SimpleVertex {
    fn into_simplevertex(self) -> SimpleVertex {
        self
    }

    fn normal(self) -> UnitVector<3> {
        self.normal
    }
}

impl TransformBy<Pose> for SimpleVertex {
    fn transform_by(&self, pose: &Pose) -> Self {
        Self {
            position: pose.transform(&self.position),
            normal: pose.transform(&self.normal),
            tangent: pose.transform(&self.tangent),
            ..*self
        }
    }
}

impl ExtendedVertex<3> for SimpleVertex {
    fn position(&self) -> Vector<3> {
        self.position
    }

    fn from_incomplete_triangle(
        incomplete_triangle: [IncompleteVertex<3>; 3],
    ) -> Option<[Self; 3]> {
        let temp_incomplete_triangle = incomplete_triangle.clone();

        let new_normal = LazyCell::new(|| {
            let ab = temp_incomplete_triangle[1].position - temp_incomplete_triangle[0].position;
            let ac = temp_incomplete_triangle[2].position - temp_incomplete_triangle[0].position;
            ab.cross(ac).try_normalize()
        });

        let new_tangent = LazyCell::new(|| {
            let ab = temp_incomplete_triangle[1].position - temp_incomplete_triangle[0].position;
            let ac = temp_incomplete_triangle[2].position - temp_incomplete_triangle[0].position;
            let z = temp_incomplete_triangle[1].texture? - temp_incomplete_triangle[0].texture?;
            let g = temp_incomplete_triangle[2].texture? - temp_incomplete_triangle[0].texture?;

            (ab.scale(g[1]) - ac.scale(z[1])) // .scale(1.0/(g[1]*z[0] - g[0]*z[1]))
                .try_normalize() // change of basis
        });

        incomplete_triangle.try_map(
            |IncompleteVertex {
                 position,
                 texture,
                 normal,
                 tangent,
                 ..
             }| {
                let final_normal = normal.or_else(|| *new_normal)?;
                let final_tangent = tangent.or_else(|| *new_tangent)?;

                Some(Self {
                    position,
                    texture: texture?,
                    normal: final_normal,
                    tangent: final_tangent,
                })
            },
        )
    }
}
