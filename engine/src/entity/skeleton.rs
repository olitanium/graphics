use graphics::linear_algebra::Vector;
use graphics::vertex::Vertex;
use utils::partition;

use crate::SimpleEntity;
use crate::entity::aggregate_entity::AggregateUnit;
use crate::entity::{FromToAggregateEntity, Mass};
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{Collision, SimpleVertex, SkeletonBb, SkeletonCubic, Volume};

#[derive(Debug)]
pub struct SkeletonEntity<T> {
    pub model: SkeletonCubic,
    pub pose: Pose,
    pub mass: Mass,
    pub(crate) bounding_box: Option<SkeletonBb<SimpleVertex, T>>,
    pub velocity: Vector<3>,
}

impl<T> Default for SkeletonEntity<T> {
    fn default() -> Self {
        Self {
            mass: Default::default(),
            pose: Default::default(),
            model: Default::default(),
            bounding_box: Default::default(),
            velocity: Default::default(),
        }
    }
}

impl<T> SkeletonEntity<T> {
    pub fn from_cubic<F: FnMut(&[(Vector<3>, [SimpleVertex; 3])]) -> T>(
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

pub fn make_bounding_box<T, F: FnMut(&[(Vector<3>, [SimpleVertex; 3])]) -> T>(
    cubic: &SkeletonCubic,
    depth: usize,
    init: F,
) -> Option<SkeletonBb<SimpleVertex, T>> {
    let per_mesh = partition(cubic.triangles(), |(bone, _)| vec![bone.0], |(_, tri)| tri);

    SkeletonBb::from_vec_triangle(per_mesh, depth, init)
}

impl<T> SkeletonEntity<T> {
    pub const fn bounding_box(&self) -> Option<&SkeletonBb<SimpleVertex, T>>
    where
        Vector<3>: Volume<3>,
        [(); Vector::<3>::ELEMENT_COUNT]:,
    {
        self.bounding_box.as_ref()
    }

    pub fn update(&mut self, timestep: f32) {
        let delta_pose = Pose::new_from_translation(self.velocity.scale(timestep));
        self.pose.push_change(delta_pose);
    }
}

impl<TThis> Collision for SkeletonEntity<TThis> {
    type FuncInputs = <SkeletonBb<SimpleVertex, TThis> as Collision>::FuncInputs;
    type Meta = <SkeletonBb<SimpleVertex, TThis> as Collision>::Meta;
    type Transform<'a> = <SkeletonBb<SimpleVertex, TThis> as Collision>::Transform<'a>;

    fn collides<
             T: Collision,
            F: FnMut(Self::FuncInputs, &mut Self::Meta, T::FuncInputs, &mut T::Meta)
    >(
        &self,
        transform: &Self::Transform<'_>,
        target: &T,
        target_transform: &T::Transform<'_>,
        alter_meta: F,
    ) -> bool {
        let Some(bbox) = &self.bounding_box else {
            return false;
        };
        bbox.collides(
            transform,
            target,
            target_transform,
            alter_meta,
        )
    }
}

impl<T> From<SimpleEntity<T>> for SkeletonEntity<T> {
    fn from(value: SimpleEntity<T>) -> Self {
        let SimpleEntity {
            model,
            pose,
            mass,
            bounding_box,
            velocity,
        } = value;

        let model = model.into();
        let bounding_box = bounding_box.map(Into::into);

        SkeletonEntity {
            model,
            pose,
            mass,
            bounding_box,
            velocity,
        }
    }
}

impl<T> FromToAggregateEntity<T> for SkeletonEntity<T> {
    type Model = SkeletonCubic;

    #[allow(private_interfaces)]
    fn from_aggregate_unit(entity: AggregateUnit<T, Self::Model>, pose: Pose) -> Self {
        let AggregateUnit {model, bounding_box, ..} = entity;
        Self { model, pose, mass: Default::default(), bounding_box, velocity: Default::default() }
    }

    fn to_pair_model_bbox(
        self,
    ) -> (
        Self::Model,
        Option<<Self::Model as super::EntityInner>::BoundingBox<T>>,
    ) {
        let Self {
            model,
            bounding_box,
            ..
        } = self;

        (model, bounding_box)
    }
}
