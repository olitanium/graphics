use std::{convert, iter};

use graphics::linear_algebra::Vector;
use utils::partition;

use crate::entity::{EntityInner, Mass, ToAggregateEntity};
use crate::modelling::cubic::SkinningCubic;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{Collision, SkeletalVertex, SkeletonBb};

#[derive(Debug, Clone)]
pub struct SkinningEntity<T> {
    pub model: SkinningCubic,
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
        model: SkinningCubic,
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
    cubic: &SkinningCubic,
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

    SkeletonBb::from_vec_triangle(per_mesh, depth, init)
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

impl<TThis> Collision for SkinningEntity<TThis> {
    type FuncInputs = <SkeletonBb<SkeletalVertex, TThis> as Collision>::FuncInputs;
    type Meta = <SkeletonBb<SkeletalVertex, TThis> as Collision>::Meta;
    type Transform<'a> = <SkeletonBb<SkeletalVertex, TThis> as Collision>::Transform<'a>;

    fn collides<
        T: Collision,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, T::FuncInputs, &mut T::Meta),
    >(
        &self,
        transform: Self::Transform<'_>,
        target: &T,
        target_transform: T::Transform<'_>,
        alter_meta: F,
    ) -> bool
    where
        for<'a> T::Transform<'a>: Clone,
    {
        let Some(bbox) = &self.bounding_box else {
            return false;
        };
        bbox.collides(transform, target, target_transform, alter_meta)
    }
}

impl<T> ToAggregateEntity<T, SkinningCubic> for SkinningEntity<T> {
    fn to_pair_model_bbox(
        self,
    ) -> (
        SkinningCubic,
        Option<<SkinningCubic as EntityInner>::BoundingBox<T>>,
    ) {
        let Self {
            model,
            bounding_box,
            ..
        } = self;

        (model, bounding_box)
    }
}
