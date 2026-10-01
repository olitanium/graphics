use core::fmt::Debug;

use geometry::Pose;
use graphics::linear_algebra::{UnitVector, Vector};

use crate::entity::physics::Physics;
use crate::entity::{EntityInner, Mass, SimplePhysics, ToAggregateEntity};
use crate::modelling::cubic::SimpleCubic;
use crate::modelling::{BoundingBox, Collision, CubicVertex, SimpleVertex, SkeletonCubic, Volume};

#[derive(Debug)]
pub struct SimpleEntity<T> {
    pub model: SimpleCubic,
    pub bounding_box: Option<BoundingBox<SimpleVertex, T>>,
    pub physics: SimplePhysics,
}

impl<T> Default for SimpleEntity<T> {
    fn default() -> Self {
        Self {
            model: SimpleCubic::default(),
            bounding_box: Option::default(),
            physics: SimplePhysics::default(),
        }
    }
}

impl<T> SimpleEntity<T> {
    pub fn from_simple_cubic<F>(model: SimpleCubic, depth: usize, mass: Mass, init: F) -> Self
    where
        F: FnMut(&[(Vector<3>, [SimpleVertex; 3])]) -> T,
    {
        let bounding_box = model.make_bounding_box(depth, init);

        let mut physics = SimplePhysics::default();
        physics.mass = mass;

        Self {
            model,
            bounding_box,
            physics,
        }
    }

    #[inline]
    pub const fn temp_as_simplecubic_and_anim(&self) -> (&SimpleCubic, Pose) {
        (&self.model, self.physics.pose)
    }
}

impl<T> SimpleEntity<T> {
    #[inline]
    pub const fn bounding_box(&self) -> Option<&BoundingBox<SimpleVertex, T>>
    where
        Vector<3>: Volume,
    {
        self.bounding_box.as_ref()
    }

    #[inline]
    pub fn move_collide_repel<B>(
        &mut self,
        other: &mut B,
        other_transform: &B::Transform<'_>,
    ) -> (Option<UnitVector<3>>, Option<UnitVector<3>>)
    where
        <B as Collision>::Vertex: CubicVertex + PartialEq + Debug,
        B: Physics,
    {
        <Self as Physics>::move_collide_repel(self, &(), other, other_transform)
    }
}

impl<TThis> Collision for SimpleEntity<TThis> {
    type FuncInputs = <BoundingBox<SimpleVertex, TThis> as Collision>::FuncInputs;
    type Meta = <BoundingBox<SimpleVertex, TThis> as Collision>::Meta;
    type Transform<'a> = ();
    type Vertex = <BoundingBox<SimpleVertex, TThis> as Collision>::Vertex;
    type VertexTransform<'a> = ();

    fn collides<T, F>(
        &self,
        _self_transform: &Self::Transform<'_>,
        _self_vertex_transform: &Self::VertexTransform<'_>,
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
            &self.physics.pose,
            &self.physics.pose,
            target,
            target_transform,
            target_vertex_transform,
            alter_meta,
        )
    }
}

impl<T> ToAggregateEntity<SimpleCubic> for SimpleEntity<T> {
    type BoundingBoxInner = T;

    fn to_pair_model_bbox(
        self,
    ) -> (
        SimpleCubic,
        Option<<SimpleCubic as EntityInner>::BoundingBox<Self::BoundingBoxInner>>,
    ) {
        let Self {
            model,
            bounding_box,
            ..
        } = self;
        (model, bounding_box)
    }

    fn root_pose(&self) -> Pose {
        self.physics.pose
    }
}

impl<T> ToAggregateEntity<SkeletonCubic> for SimpleEntity<T> {
    type BoundingBoxInner = T;

    fn to_pair_model_bbox(
        self,
    ) -> (
        SkeletonCubic,
        Option<<SkeletonCubic as EntityInner>::BoundingBox<Self::BoundingBoxInner>>,
    ) {
        let Self {
            model,
            bounding_box,
            ..
        } = self;
        (model.into(), bounding_box.map(Into::into))
    }

    fn root_pose(&self) -> Pose {
        self.physics.pose
    }
}

impl<T> Physics for SimpleEntity<T> {
    fn physics(&mut self) -> Option<&mut SimplePhysics> {
        Some(&mut self.physics)
    }
}
