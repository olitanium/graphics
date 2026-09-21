use engine::SimpleEntity;
use engine::entity::{EntityInner, ToAggregateEntity};
use engine::modelling::cubic::SimpleCubic;
use engine::modelling::{Collision, SkeletonCubic};

pub struct Bear {
    pub inner: SimpleEntity<()>,
}

impl Collision for Bear {
    type FuncInputs = <SimpleEntity<()> as Collision>::FuncInputs;
    type Meta = <SimpleEntity<()> as Collision>::Meta;
    type Transform<'a> = <SimpleEntity<()> as Collision>::Transform<'a>;
    type Vertex = <SimpleEntity<()> as Collision>::Vertex;
    type VertexTransform<'a> = <SimpleEntity<()> as Collision>::VertexTransform<'a>;

    fn collides<
        T: Collision + ?Sized,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, T::FuncInputs, &mut T::Meta) -> bool,
    >(
        &self,
        self_transform: &Self::Transform<'_>,
        self_vertex_transform: &Self::VertexTransform<'_>,
        target: &T,
        target_transform: &T::Transform<'_>,
        target_vertex_transform: &T::VertexTransform<'_>,
        alter_meta: F,
    ) -> Option<Vec<([Self::Vertex; 3], [T::Vertex; 3])>> {
        self.inner.collides(
            self_transform,
            self_vertex_transform,
            target,
            target_transform,
            target_vertex_transform,
            alter_meta,
        )
    }
}

impl ToAggregateEntity<SimpleCubic> for Bear {
    type BoundingBoxInner = ();

    fn to_pair_model_bbox(
        self,
    ) -> (
        SimpleCubic,
        Option<<SimpleCubic as EntityInner>::BoundingBox<()>>,
    ) {
        self.inner.to_pair_model_bbox()
    }

    fn root_pose(&self) -> engine::geometry::Pose {
        <SimpleEntity<()> as ToAggregateEntity<SimpleCubic>>::root_pose(&self.inner)
    }
}

impl ToAggregateEntity<SkeletonCubic> for Bear {
    type BoundingBoxInner = ();

    fn to_pair_model_bbox(
        self,
    ) -> (
        SkeletonCubic,
        Option<<SkeletonCubic as engine::entity::EntityInner>::BoundingBox<()>>,
    ) {
        self.inner.to_pair_model_bbox()
    }

    fn root_pose(&self) -> engine::geometry::Pose {
        <SimpleEntity<()> as ToAggregateEntity<SkeletonCubic>>::root_pose(&self.inner)
    }
}
