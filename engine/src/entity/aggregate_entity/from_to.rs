use crate::entity::EntityInner;
use crate::entity::aggregate_entity::AggregateUnit;
use crate::modelling::CubicVertex;
use crate::modelling::cubic::geometry::Pose;

pub trait ToAggregateEntity<T, M: EntityInner>
where
    M::Vertex: CubicVertex,
{
    fn to_pair_model_bbox(self) -> (M, Option<M::BoundingBox<T>>);
}
