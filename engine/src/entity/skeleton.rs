use geometry::Pose;
use graphics::linear_algebra::Vector;
use graphics::vertex::Vertex;

use crate::SimpleEntity;
use crate::entity::{Mass, ToAggregateEntity};
use crate::modelling::cubic::CalculatedAnimation;
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
    let per_mesh = partition::partition(
        &cubic.triangles(),
        |&&(bone, _)| vec![bone.0],
        |&&(_, tri)| tri,
    );

    SkeletonBb::from_vec_triangle(per_mesh, depth, init)
}

impl<T> SkeletonEntity<T> {
    pub const fn bounding_box(&self) -> Option<&SkeletonBb<SimpleVertex, T>>
    where
        Vector<3>: Volume<3>,
        [(); <Vector<3> as Vertex>::ELEMENT_COUNT]:,
    {
        self.bounding_box.as_ref()
    }

    pub fn update(&mut self, timestep: f32) {
        let delta_pose = Pose::new_from_translation(self.velocity.scale(timestep));
        self.pose.push_change(delta_pose);
    }

    pub fn get_pair_model_bind_pose(&self) -> (&SkeletonCubic, CalculatedAnimation) {
        let animation = CalculatedAnimation::bind_pose(&self.model, self.pose);
        (&self.model, animation)
    }
}

impl<TThis> Collision for SkeletonEntity<TThis> {
    type FuncInputs = <SkeletonBb<SimpleVertex, TThis> as Collision>::FuncInputs;
    type Meta = <SkeletonBb<SimpleVertex, TThis> as Collision>::Meta;
    type Transform<'a> = &'a CalculatedAnimation;
    type Vertex = <SkeletonBb<SimpleVertex, TThis> as Collision>::Vertex;

    fn collides<
        'a,
        'b,
        T: Collision,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, T::FuncInputs, &mut T::Meta),
    >(
        &'a self,
        transform: Self::Transform<'_>,
        target: &'b T,
        target_transform: T::Transform<'_>,
        alter_meta: F,
    ) -> Option<Vec<(&'a [Self::Vertex; 3], &'b [T::Vertex; 3])>>
    where
        for<'c> T::Transform<'c>: Clone,
    {
        // let Some(bbox) = &self.bounding_box else {
        // return None;
        // };
        self.bounding_box.as_ref()?.collides(
            (&self.model.skeleton, transform),
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
            bounding_box,
            physics,
        } = value;

        let model = model.into();
        let bounding_box = bounding_box.map(Into::into);

        SkeletonEntity {
            model,
            pose: *physics.pose(),
            mass: *physics.mass(),
            bounding_box,
            velocity: *physics.velocity(),
        }
    }
}

impl<T> ToAggregateEntity<T, SkeletonCubic> for SkeletonEntity<T> {
    fn to_pair_model_bbox(
        self,
    ) -> (
        SkeletonCubic,
        Option<<SkeletonCubic as super::EntityInner>::BoundingBox<T>>,
    ) {
        let Self {
            model,
            bounding_box,
            ..
        } = self;

        (model, bounding_box)
    }
}
