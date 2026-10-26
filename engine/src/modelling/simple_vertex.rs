use core::{gca, mem};

use geometry::{Pose, Transform as _, TransformBy};
use graphics::linear_algebra::{UnitVector, Vector};
use graphics::types::VertexAttrType;
use graphics::vertex::Vertex;
use utils::error_boilerplate;

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
    const ELEMENT_COUNT: usize = gca!(4);

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

#[derive(Debug)]
pub enum FromIncompleteVertexError {
    Normal(Box<[IncompleteVertex<3>; 3]>),
    TextureCoords(Box<[IncompleteVertex<3>; 3]>),
    Tangent(Box<[IncompleteVertex<3>; 3]>),
}
error_boilerplate!(FromIncompleteVertexError);

impl ExtendedVertex for SimpleVertex {
    type FromIncompleteVertexError = FromIncompleteVertexError;
    type PreferredTransform<'a> = Pose;

    const LEN: usize = gca!(3);

    fn position(&self) -> Vector<3> {
        self.position
    }

    fn from_incomplete_triangle(
        triangle: [IncompleteVertex<3>; 3],
    ) -> Result<[Self; 3], Self::FromIncompleteVertexError> {
        let [first, second, third] = triangle.each_ref();

        let new_normal = || {
            let ab = second.position - first.position;
            let ac = third.position - first.position;
            let out = ab.cross(ac).try_normalize();
            out.ok_or_else(|| FromIncompleteVertexError::Normal(Box::new(triangle.clone())))
        };

        let new_tangent = || {
            let ab = second.position - first.position;
            let ac = third.position - first.position;
            let z1 = try { (second.texture? - first.texture?).y() }.ok_or_else(|| {
                FromIncompleteVertexError::TextureCoords(Box::new(triangle.clone()))
            })?;
            let g1 = try { (third.texture? - first.texture?).y() }.ok_or_else(|| {
                FromIncompleteVertexError::TextureCoords(Box::new(triangle.clone()))
            })?;

            let out = (ab.scale(g1) - ac.scale(z1)).try_normalize();

            out.ok_or_else(|| FromIncompleteVertexError::Tangent(Box::new(triangle.clone())))
        };

        triangle.clone().try_map(
            |IncompleteVertex {
                 position,
                 texture,
                 normal,
                 tangent,
                 ..
             }| {
                let final_normal = normal.map_or_else(new_normal, Ok)?;
                let final_tangent = tangent.map_or_else(new_tangent, Ok)?;

                Ok(Self {
                    position,
                    texture: texture.ok_or_else(|| {
                        FromIncompleteVertexError::Tangent(Box::new(triangle.clone()))
                    })?,
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

#[cfg(test)]
mod tests {
    use geometry::{Pose, TransformBy};
    use graphics::linear_algebra::{UnitVector, vector};

    use crate::modelling::{ExtendedVertex as _, IncompleteVertex, SimpleVertex};

    #[test]
    fn incomplete() {
        let triangle = [
            IncompleteVertex::new(vector![0.0, 0.0, 0.0]).texture(vector![0.0, 0.0]),
            IncompleteVertex::new(vector![1.0, 0.0, 0.0]).texture(vector![1.0, 0.0]),
            IncompleteVertex::new(vector![0.0, 1.0, 0.0]).texture(vector![0.0, 1.0]),
        ];

        for pose in [
            Pose::IDENTITY,
            Pose::new_from_to_up(
                vector![0.0, 1.0, 0.0],
                vector![0.0, 0.0, 1.0],
                vector![1.0, 0.0, 0.0].try_normalize().unwrap(),
            )
            .unwrap(),
            Pose::new_from_to_up(
                vector![2.0, 1.0, 0.0],
                vector![0.0, 1.0, 1.0],
                vector![1.0, 0.0, 4.0].try_normalize().unwrap(),
            )
            .unwrap(),
        ] {
            let simple_triangle =
                SimpleVertex::from_incomplete_triangle(triangle.clone().transform_by(pose))
                    .unwrap();

            assert_eq!(simple_triangle[0].normal, UnitVector::Z.transform_by(pose));
            assert_eq!(simple_triangle[0].tangent, UnitVector::X.transform_by(pose));
        }
    }
}
