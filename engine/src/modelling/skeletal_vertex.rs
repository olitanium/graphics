use std::array;
use std::mem::offset_of;

use graphics::linear_algebra::Vector;
use graphics::types::VertexAttrType;
use graphics::vertex::Vertex;

use crate::TransformBy;
use crate::modelling::advanced_vertex::IncompleteVertex;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{CubicVertex, ExtendedVertex, SimpleVertex};

#[derive(Debug, Clone, Copy)]
pub struct SkeletalVertex {
    simplevertex: SimpleVertex,
    bone_indices: [f32; 4],
    bone_weights: [f32; 4],
}

impl Vertex for SkeletalVertex {
    const ELEMENT_COUNT: usize = SimpleVertex::ELEMENT_COUNT + 2;

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        let simple_vertex = SimpleVertex::offsets();
        let simple_vertex_len = SimpleVertex::ELEMENT_COUNT;
        let mut out = [0; Self::ELEMENT_COUNT];
        for (index, element) in simple_vertex.into_iter().enumerate() {
            out[index] = offset_of!(SkeletalVertex, simplevertex) + element;
        }

        out[simple_vertex_len] = offset_of!(SkeletalVertex, bone_indices);
        out[simple_vertex_len + 1] = offset_of!(SkeletalVertex, bone_weights);

        out
    }

    fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT] {
        let simple_vertex = SimpleVertex::types_of();
        let simple_vertex_len = SimpleVertex::ELEMENT_COUNT;
        let mut out = [(VertexAttrType::f32, 0); Self::ELEMENT_COUNT];
        for (index, element) in simple_vertex.into_iter().enumerate() {
            out[index] = element;
        }

        const MAX_BONES_PER_VERTEX: usize = 4;

        out[simple_vertex_len] = (VertexAttrType::f32, MAX_BONES_PER_VERTEX);
        out[simple_vertex_len + 1] = (VertexAttrType::f32, MAX_BONES_PER_VERTEX);

        out
    }
}

impl ExtendedVertex<3> for SkeletalVertex {
    fn position(&self) -> Vector<3> {
        self.simplevertex.position
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
                simplevertex: simple_vertex[index],
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
            simplevertex: self.simplevertex.transform_by(operator),
            bone_indices: self.bone_indices,
            bone_weights: self.bone_weights,
        }
    }
}

impl CubicVertex for SkeletalVertex {}
