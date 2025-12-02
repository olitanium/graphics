use std::{convert, iter};

use graphics::linear_algebra::Vector;
use utils::partition;

use crate::entity::Mass;
use crate::modelling::cubic::SkeletonCubic;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{SkeletalVertex, SkeletonBb};

#[derive(Debug, Clone)]
pub struct SkinningEntity<T> {
    pub model: SkeletonCubic,
    pub pose: Pose,
    pub mass: Mass,
    pub(crate) bounding_box: Option<SkeletonBb<SkeletalVertex, T>>,
    pub velocity: Vector<3>,
}

// TODO: investigate whether this can be derived. There's a problem re: the `T`
impl<T> Default for SkinningEntity<T> {
    fn default() -> Self {
        Self {
            model: Default::default(),
            pose: Default::default(),
            mass: Default::default(),
            bounding_box: Default::default(),
            velocity: Default::default(),
        }
    }
}

impl<T> SkinningEntity<T> {
    pub fn from_skeletoncubic<F: FnMut(&[(Vector<3>, [SkeletalVertex; 3])]) -> T>(
        model: SkeletonCubic,
        depth: usize,
        mass: Mass,
        init: F,
    ) -> Option<Self> {
        let bounding_box = make_bounding_box(&model, depth, init);

        Some(Self {
            model,
            mass,
            bounding_box,
            ..Self::default()
        })
    }
}

pub fn make_bounding_box<T, F: FnMut(&[(Vector<3>, [SkeletalVertex; 3])]) -> T>(
    cubic: &SkeletonCubic,
    depth: usize,
    init: F,
) -> Option<SkeletonBb<SkeletalVertex, T>> {
    let per_mesh = partition(
        cubic.triangles(),
        |tri| {
            tri.iter()
                .flat_map(|vertex| {
                    iter::zip(vertex.bone_indices, vertex.bone_weights)
                        .filter_map(|(index, weight)| (weight != 0.0).then_some(index as usize))
                })
                .collect()
        },
        convert::identity,
    );

    SkeletonBb::from_vec_triangle(per_mesh, depth, cubic.skeleton.clone(), init)
}

impl<T> SkinningEntity<T> {
    pub const fn bounding_box(&self) -> Option<&SkeletonBb<SkeletalVertex, T>> {
        self.bounding_box.as_ref()
    }

    pub fn update(&mut self, timestep: f32) {
        let delta_pose = Pose::new_from_translation(self.velocity.scale(timestep));
        self.pose.push_change(delta_pose);
    }
}
