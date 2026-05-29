use engine::entity::{SkeletonEntity, ToAggregateEntity};
use engine::modelling::cubic::lighting::shadow::{ShadowListLights, ShadowPointLight};

pub struct Sun {
    pub inner: SkeletonEntity<()>,
    pub light: ShadowPointLight,
}

// impl Collision for Morgan {
// type FuncInputs = <SimpleEntity<()> as Collision>::FuncInputs;
// type Meta = <SimpleEntity<()> as Collision>::Meta;
// type Transform<'a> = <SimpleEntity<()> as Collision>::Transform<'a>;
//
// fn collides<
// T: Collision,
// F: FnMut(Self::FuncInputs, &mut Self::Meta, T::FuncInputs, &mut T::Meta),
// >(
// &self,
// transform: Self::Transform<'_>,
// target: &T,
// target_transform: T::Transform<'_>,
// alter_meta: F,
// ) -> bool
// where
// for<'a> T::Transform<'a>: Clone,
// {
// self.inner
// .collides(transform, target, target_transform, alter_meta)
// }
// }
//
// impl ToAggregateEntity<(), SimpleCubic> for Morgan {
// fn to_pair_model_bbox(
// self,
// ) -> (
// SimpleCubic,
// Option<<SimpleCubic as engine::entity::EntityInner>::BoundingBox<()>>,
// ) {
// self.inner.to_pair_model_bbox()
// }
// }
//
// impl ToAggregateEntity<(), SkeletonCubic> for Morgan {
// fn to_pair_model_bbox(
// self,
// ) -> (
// SkeletonCubic,
// Option<<SkeletonCubic as engine::entity::EntityInner>::BoundingBox<()>>,
// ) {
// self.inner.to_pair_model_bbox()
// }
// }
