use graphics::linear_algebra::Vector;

use crate::SimpleEntity;
use crate::entity::physics::Physics;
use crate::entity::{Mass, SimplePhysics, ToAggregateEntity};
use crate::modelling::cubic::CalculatedAnimation;
use crate::modelling::{Collision, SimpleVertex, SkeletonBb, SkeletonCubic};

#[derive(Debug)]
pub struct SkeletonEntity<T> {
    pub model: SkeletonCubic,
    pub(crate) bounding_box: Option<SkeletonBb<SimpleVertex, T>>,
    pub physics: SimplePhysics,
}

impl<T> Default for SkeletonEntity<T> {
    fn default() -> Self {
        Self {
            model: SkeletonCubic::default(),
            bounding_box: Option::default(),
            physics: SimplePhysics::default(),
        }
    }
}

impl<T> SkeletonEntity<T> {
    pub fn from_cubic<F>(model: SkeletonCubic, depth: usize, mass: Mass, init: F) -> Option<Self>
    where
        F: FnMut(&[(Vector<3>, [SimpleVertex; 3])]) -> T,
    {
        let bounding_box = make_bounding_box(&model, depth, init);

        let mut physics = SimplePhysics::default();
        physics.mass = mass;

        Some(Self {
            model,
            bounding_box,
            physics,
        })
    }
}

pub fn make_bounding_box<T, F>(
    cubic: &SkeletonCubic,
    depth: usize,
    init: F,
) -> Option<SkeletonBb<SimpleVertex, T>>
where
    F: FnMut(&[(Vector<3>, [SimpleVertex; 3])]) -> T,
{
    let per_mesh = partition::partition(
        &cubic.triangles(),
        |&&(bone, _)| vec![bone.0],
        |&&(_, tri)| tri,
    );

    SkeletonBb::from_vec_triangle(per_mesh, depth, init)
}

impl<T> SkeletonEntity<T> {
    pub const fn bounding_box(&self) -> Option<&SkeletonBb<SimpleVertex, T>> {
        self.bounding_box.as_ref()
    }

    pub fn debug_get_pair_model_bind_pose(&self) -> (&SkeletonCubic, CalculatedAnimation) {
        let animation = CalculatedAnimation::bind_pose(&self.model, self.physics.pose);
        (&self.model, animation)
    }
}

impl<TThis> Collision for SkeletonEntity<TThis> {
    type FuncInputs = <SkeletonBb<SimpleVertex, TThis> as Collision>::FuncInputs;
    type Meta = <SkeletonBb<SimpleVertex, TThis> as Collision>::Meta;
    type Transform<'a> = CalculatedAnimation;
    type Vertex = <SkeletonBb<SimpleVertex, TThis> as Collision>::Vertex;
    type VertexTransform<'a> = ();

    fn collides<T, F>(
        &self,
        self_transform: &Self::Transform<'_>,
        _self_vertex_transform: &(),
        target: &T,
        target_transform: &T::Transform<'_>,
        target_vertex_transform: &T::VertexTransform<'_>,
        alter_meta: F,
    ) -> Option<Vec<([Self::Vertex; 3], [T::Vertex; 3])>>
    where
        T: Collision + ?Sized,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, T::FuncInputs, &mut T::Meta) -> bool,
    {
        self.bounding_box.as_ref()?.collides(
            &(&self.model.skeleton, self_transform),
            &(),
            target,
            target_transform,
            target_vertex_transform,
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

        Self {
            model,
            bounding_box,
            physics,
        }
    }
}

impl<T> ToAggregateEntity<SkeletonCubic> for SkeletonEntity<T> {
    type BoundingBoxInner = T;

    fn to_pair_model_bbox(
        self,
    ) -> (
        SkeletonCubic,
        Option<<SkeletonCubic as super::EntityInner>::BoundingBox<Self::BoundingBoxInner>>,
    ) {
        let Self {
            model,
            bounding_box,
            ..
        } = self;

        (model, bounding_box)
    }

    fn root_pose(&self) -> geometry::Pose {
        self.physics.pose
    }
}

impl<T> Physics for SkeletonEntity<T> {
    fn physics(&mut self) -> Option<&mut SimplePhysics> {
        Some(&mut self.physics)
    }
}
