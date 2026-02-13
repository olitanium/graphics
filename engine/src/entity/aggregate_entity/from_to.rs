use graphics::Model;

use crate::entity::EntityInner;
use crate::entity::aggregate_entity::AggregateUnit;
use crate::modelling::CubicVertex;
use crate::modelling::cubic::geometry::Pose;
pub trait FromToAggregateEntity<T>
where
    <<Self as FromToAggregateEntity<T>>::Model as Model>::Vertex: CubicVertex,
{
    type Model: EntityInner;

    #[doc(hidden)]
    #[allow(private_interfaces)]
    fn from_aggregate_unit(entity: AggregateUnit<T, Self::Model>, pose: Pose) -> Self;

    #[doc(hidden)]
    fn to_pair_model_bbox(
        self,
    ) -> (
        Self::Model,
        Option<<Self::Model as EntityInner>::BoundingBox<T>>,
    );
}

// impl<T> FromToAggregateEntity<T> for SimpleEntity<T> {
// type Model = SkeletonCubic;
// }
