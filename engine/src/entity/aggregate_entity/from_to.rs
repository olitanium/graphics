use crate::entity::EntityInner;
use crate::modelling::CubicVertex;

pub trait ToAggregateEntity<T, M: EntityInner>
where
    M::Vertex: CubicVertex,
{
    fn to_pair_model_bbox(self) -> (M, Option<M::BoundingBox<T>>);
}
