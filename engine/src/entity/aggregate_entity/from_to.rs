use crate::entity::EntityInner;
use crate::modelling::CubicVertex;

pub trait ToAggregateEntity<M: EntityInner>
where
    M::Vertex: CubicVertex,
{
    type BoundingBoxInner;
    fn to_pair_model_bbox(self) -> (M, Option<M::BoundingBox<Self::BoundingBoxInner>>);
}
