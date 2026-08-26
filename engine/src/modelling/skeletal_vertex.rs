use std::mem::offset_of;
use std::{array, iter};

use dual_number::DualQuaternion;
use geometry::{Pose, TransformBy};
use graphics::linear_algebra::{UnitVector, Vector};
use graphics::types::VertexAttrType;
use graphics::vertex::Vertex;

use crate::modelling::cubic::CalculatedAnimation;
use crate::modelling::extended_vertex::IncompleteVertex;
use crate::modelling::{CubicVertex, ExtendedVertex, SimpleVertex, Skeleton};

#[derive(Debug, Clone, Copy)]
pub struct SkeletalVertex {
    pub position: Vector<3>,
    pub texture: Vector<2>,
    pub normal: UnitVector<3>,
    pub tangent: UnitVector<3>,
    pub bone_indices: [f32; 4],
    pub bone_weights: [f32; 4],
}

impl Vertex for SkeletalVertex {
    type const ELEMENT_COUNT: usize = 6;

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

    fn types_of() -> [VertexAttrType; Self::ELEMENT_COUNT] {
        let [vec2] = Vector::<2>::types_of();
        let [vec3] = Vector::<3>::types_of();
        let [uvec3] = UnitVector::<3>::types_of();
        let [f32x4] = <[f32; 4]>::types_of();
        [vec3, vec2, uvec3, uvec3, f32x4, f32x4]
    }
}

impl<'a> TransformBy<(&'a Skeleton, &'a CalculatedAnimation)> for SkeletalVertex {
    fn transform_by(
        &self,
        &(skeleton, animation): &(&'a Skeleton, &'a CalculatedAnimation),
    ) -> Self {
        let mut accum = DualQuaternion::ZERO;
        let bones = animation.poses(skeleton).unwrap();
        for (weight, index) in iter::zip(self.bone_weights, self.bone_indices) {
            let index = index as usize;
            if weight == 0.0 {
                break;
            }
            let bone = bones.get(index).unwrap();
            let dual_quat = bone.into_inner().u();
            accum = accum + dual_quat;
        }

        let unit_pose = accum.normalize();
        let pose = Pose::new_from_dual(unit_pose);

        Self {
            position: self.position.transform_by(&pose),
            normal: self.normal.transform_by(&pose),
            tangent: self.tangent.transform_by(&pose),
            ..*self
        }
    }
}

impl ExtendedVertex for SkeletalVertex {
    type PreferredTransform<'a> = (&'a Skeleton, &'a CalculatedAnimation);

    type const LEN: usize = 3;

    fn position(&self) -> Vector<3> {
        self.position
    }

    #[inline]
    fn from_incomplete_triangle(mut triangle: [IncompleteVertex<3>; 3]) -> Option<[Self; 3]> {
        let weights = triangle.each_mut().map(|vert| {
            let weight = vert.weights.take();

            const NO_WEIGHT: (usize, f32) = (0, 0.0);
            const ONLY_ROOT: (usize, f32) = (0, 1.0);
            weight.map_or([ONLY_ROOT, NO_WEIGHT, NO_WEIGHT, NO_WEIGHT], |vec| {
                // while vec.push((0, 0.0)).is_ok() {}
                // vec.try_into_array().expect("filled above")
                vec.into_array_with(|| NO_WEIGHT)
            })
        });

        let simple_vertex = SimpleVertex::from_incomplete_triangle(triangle)?;

        Some(array::from_fn(move |index| Self {
            position: simple_vertex[index].position,
            texture: simple_vertex[index].texture,
            normal: simple_vertex[index].normal,
            tangent: simple_vertex[index].tangent,
            bone_weights: weights[index].map(|(_, weight)| weight),
            bone_indices: weights[index].map(|(index, _)| index as f32),
        }))
    }
}

// impl TransformBy<Pose> for SkeletalVertex {
// fn transform_by(&self, operator: &Pose) -> Self {
// Self {
// position: self.position.transform_by(operator),
// normal: self.normal.transform_by(operator),
// texture: self.texture,
// tangent: self.tangent.transform_by(operator),
// bone_indices: self.bone_indices,
// bone_weights: self.bone_weights,
// }
// }
// }

impl CubicVertex for SkeletalVertex {}
