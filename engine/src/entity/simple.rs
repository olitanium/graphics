use std::fmt::Debug;

use geometry::Pose;
use graphics::linear_algebra::{UnitVector, Vector};

use crate::entity::physics::Physics;
use crate::entity::{EntityInner, Mass, SimplePhysics, ToAggregateEntity};
use crate::modelling::cubic::SimpleCubic;
use crate::modelling::{BoundingBox, Collision, CubicVertex, SimpleVertex, SkeletonCubic, Volume};

#[derive(Debug)]
pub struct SimpleEntity<T> {
    pub model: SimpleCubic,
    pub bounding_box: Option<BoundingBox<SimpleVertex, 3, T>>,
    pub physics: SimplePhysics,
}

impl<T> Default for SimpleEntity<T> {
    fn default() -> Self {
        Self {
            model: Default::default(),
            bounding_box: Default::default(),
            physics: Default::default(),
        }
    }
}

impl<T> SimpleEntity<T> {
    pub fn from_simple_cubic<F: FnMut(&[(Vector<3>, [SimpleVertex; 3])]) -> T>(
        model: SimpleCubic,
        depth: usize,
        mass: Mass,
        init: F,
    ) -> Self {
        let bounding_box = model.make_bounding_box(depth, init);

        let mut physics = SimplePhysics::default();
        physics.mass = mass;

        Self {
            model,
            bounding_box,
            physics,
            ..Self::default()
        }
    }

    pub const fn temp_as_simplecubic_and_anim(&self) -> (&SimpleCubic, Pose) {
        (&self.model, self.physics.pose)
    }
}

impl<T> SimpleEntity<T> {
    pub const fn bounding_box(&self) -> Option<&BoundingBox<SimpleVertex, 3, T>>
    where
        Vector<3>: Volume<3>,
    {
        self.bounding_box.as_ref()
    }

    pub fn move_collide_repel<B: Physics>(
        &mut self,
        other: &mut B,
        other_transform: B::Transform<'_>,
    ) -> (Option<UnitVector<3>>, Option<UnitVector<3>>)
    where
        <B as Collision>::Vertex: CubicVertex + PartialEq,
    {
        <Self as Physics>::move_collide_repel(self, (), other, other_transform)
    }
}

impl<TThis> Collision for SimpleEntity<TThis> {
    type FuncInputs = <BoundingBox<SimpleVertex, 3, TThis> as Collision>::FuncInputs;
    type Meta = <BoundingBox<SimpleVertex, 3, TThis> as Collision>::Meta;
    type Transform<'a> = ();
    type Vertex = <BoundingBox<SimpleVertex, 3, TThis> as Collision>::Vertex;

    fn collides<
        'a,
        'b,
        T: Collision + ?Sized,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, T::FuncInputs, &mut T::Meta),
    >(
        &'a self,
        (): Self::Transform<'_>,
        target: &'b T,
        target_transform: T::Transform<'_>,
        alter_meta: F,
    ) -> Option<Vec<([Self::Vertex; 3], [T::Vertex; 3])>> {
        self.bounding_box.as_ref()?.collides(
            self.physics.pose,
            target,
            target_transform,
            alter_meta,
        )
    }
}

impl<T> ToAggregateEntity<T, SimpleCubic> for SimpleEntity<T> {
    fn to_pair_model_bbox(
        self,
    ) -> (
        SimpleCubic,
        Option<<SimpleCubic as EntityInner>::BoundingBox<T>>,
    ) {
        let Self {
            model,
            bounding_box,
            ..
        } = self;
        (model, bounding_box)
    }
}

impl<T> ToAggregateEntity<T, SkeletonCubic> for SimpleEntity<T> {
    fn to_pair_model_bbox(
        self,
    ) -> (
        SkeletonCubic,
        Option<<SkeletonCubic as EntityInner>::BoundingBox<T>>,
    ) {
        let Self {
            model,
            bounding_box,
            ..
        } = self;
        (model.into(), bounding_box.map(Into::into))
    }
}

impl<T> Physics for SimpleEntity<T> {
    fn physics(&mut self) -> Option<&mut SimplePhysics> {
        Some(&mut self.physics)
    }
}
