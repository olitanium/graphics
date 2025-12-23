use graphics::Model;

use crate::{SimpleEntity, entity::{AggregateEntity, EntityInner}, modelling::{CubicVertex, SimpleCubic}};

pub trait FromToAggregateEntity<T> where <<Self as FromToAggregateEntity<T>>::Model as Model>::Vertex: CubicVertex {
    type Model: EntityInner;

    fn from_aggregate_entity(entity: AggregateEntity<T, Self::Model>) -> Self;
    fn to_pair_model_bbox(self) -> (Self::Model, <Self::Model as EntityInner>::BoundingBox<T>);
}

/*impl<T> FromToAggregateEntity<T> for SimpleEntity<T> {
    type Model = SimpleCubic;
}*/