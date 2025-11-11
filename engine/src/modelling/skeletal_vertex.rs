use std::array;
use std::mem::offset_of;

use graphics::linear_algebra::{UnitVector, Vector};
use graphics::types::VertexAttrType;
use graphics::vertex::Vertex;

use crate::TransformBy;
use crate::modelling::advanced_vertex::IncompleteVertex;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{CubicVertex, ExtendedVertex, SimpleVertex};

#[derive(Debug, Clone, Copy)]
pub struct SkeletalVertex {
    pub position: Vector<3>,
    // 1
    pub texture: Vector<2>,
    // 2
    pub normal: UnitVector<3>,
    // 3
    pub tangent: UnitVector<3>,
    bone_indices: [f32; 4],
    bone_weights: [f32; 4],
}

impl Vertex for SkeletalVertex {
    const ELEMENT_COUNT: usize = 6;

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        [
            offset_of!(Self, position),
            offset_of!(Self, texture),
            offset_of!(Self, normal),
            offset_of!(Self, tangent),
            offset_of!(Self, bone_indices),
            offset_of!(Self, bone_weights),
        ]
    }

    fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT] {
        [
            (VertexAttrType::f32, 3),
            (VertexAttrType::f32, 2),
            (VertexAttrType::f32, 3),
            (VertexAttrType::f32, 3),
            (VertexAttrType::f32, 4),
            (VertexAttrType::f32, 4),
        ]
    }
}

impl ExtendedVertex<3> for SkeletalVertex {
    fn position(&self) -> Vector<3> {
        self.position
    }

    fn from_incomplete_triangle(mut triangle: [IncompleteVertex<3>; 3]) -> Option<[Self; 3]> {
        let weights = triangle.each_mut().map(|vert| {
            let weight = vert.weights.take();

            weight.map_or([(0, 1.0), (0, 0.0), (0, 0.0), (0, 0.0)], |mut vec| {
                while let Ok(_) = vec.push_within_capacity((0, 0.0)) {}
                vec.try_into_array().expect("filled above")
            })
        });

        let simple_vertex = SimpleVertex::from_incomplete_triangle(triangle)?;

        Some(array::from_fn(move |index| {
            Self {
                position: simple_vertex[index].position,
                texture: simple_vertex[index].texture,
                normal: simple_vertex[index].normal,
                tangent: simple_vertex[index].tangent,
                bone_weights: weights[index].map(|(_, weight)| weight),
                bone_indices: weights[index].map(
                    |(index, _)| index as f32, // TODO: make ints work here
                ),
            }
        }))
    }
}

impl TransformBy<Pose> for SkeletalVertex {
    fn transform_by(&self, operator: &Pose) -> Self {
        Self {
            position: self.position.transform_by(operator),
            normal: self.normal.transform_by(operator),
            texture: self.texture,
            tangent: self.tangent.transform_by(operator),
            bone_indices: self.bone_indices,
            bone_weights: self.bone_weights,
        }
    }
}

impl CubicVertex for SkeletalVertex {}