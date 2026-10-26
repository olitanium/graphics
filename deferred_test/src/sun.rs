use engine::SimpleEntity;
use engine::entity::SkeletonEntity;
use engine::geometry::Pose;
use engine::modelling::cubic::SimpleCubic;
use engine::modelling::cubic::lighting::shadow::ShadowPointLight;
use engine::modelling::cubic::lighting::simple::PointLight;

pub struct Sun {
    pub inner: SimpleEntity<()>,
    pub light: PointLight,
}

impl Sun {
    pub const fn model_and_pose(&self) -> (&SimpleCubic, Pose) {
        self.inner.temp_as_simplecubic_and_anim()
    }
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
