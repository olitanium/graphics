use core::mem;

use geometry::{Pose, Transform as _, TransformBy};
use graphics::linear_algebra::{UnitVector, Vector};
use graphics::types::VertexAttrType;
use graphics::vertex::Vertex;

use crate::modelling::extended_vertex::{ExtendedVertex, IncompleteVertex};
use crate::modelling::{CubicVertex, VertexHasTangent};

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

impl TransformBy<Pose> for SimpleVertex {
    fn transform_by(self, operator: Pose) -> Self {
        Self {
            position: operator.transform(self.position),
            normal: operator.transform(self.normal),
            tangent: operator.transform(self.tangent),
            ..self
        }
    }
}

impl ExtendedVertex for SimpleVertex {
    type PreferredTransform<'a> = Pose;

    type const LEN: usize = 3;

    fn position(&self) -> Vector<3> {
        self.position
    }

    fn from_incomplete_triangle(triangle: [IncompleteVertex<3>; 3]) -> Option<[Self; 3]> {
        let temp_incomplete_triangle = triangle.clone();

        let new_normal = || {
            let ab = temp_incomplete_triangle[1].position - temp_incomplete_triangle[0].position;
            let ac = temp_incomplete_triangle[2].position - temp_incomplete_triangle[0].position;
            let out = ab.cross(ac).try_normalize();
            if out.is_none() {
                eprintln!("failed to make normal: ab: {ab:?}, ac: {ac:?}");
            }
            out
        };

        let new_tangent = || {
            let ab = temp_incomplete_triangle[1].position - temp_incomplete_triangle[0].position;
            let ac = temp_incomplete_triangle[2].position - temp_incomplete_triangle[0].position;
            let z1 =
                (temp_incomplete_triangle[1].texture? - temp_incomplete_triangle[0].texture?).y();
            let g1 =
                (temp_incomplete_triangle[2].texture? - temp_incomplete_triangle[0].texture?).y();

            let out = (ab.scale(g1) - ac.scale(z1)) // .scale(1.0/(g[1]*z[0] - g[0]*z[1]))
                .try_normalize(); // change of basis

            if out.is_none() {
                eprintln!("failed to make tangent: ab: {ab:?}, ac: {ac:?}, z[1]: {z1}, g[1]: {g1}");
            }

            out
        };

        triangle.try_map(
            |IncompleteVertex {
                 position,
                 texture,
                 normal,
                 tangent,
                 ..
             }| {
                let final_normal = normal.or_else(new_normal)?;
                let final_tangent = tangent.or_else(new_tangent)?;

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

impl CubicVertex for SimpleVertex {
    fn triangle_normal(triangle: &[Self; 3]) -> UnitVector<3> {
        Self::triangle_normal_tangent(triangle)
    }

    fn multi_normal(triangle: &[Self; 3]) -> [UnitVector<3>; 3] {
        Self::triangle_multi_normal_tangent(triangle)
    }
}

impl VertexHasTangent for SimpleVertex {
    fn into_simplevertex(self) -> SimpleVertex {
        self
    }

    fn normal(&self) -> UnitVector<3> {
        self.normal
    }
}
