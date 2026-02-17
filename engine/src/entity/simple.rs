use graphics::linear_algebra::{UnitVector, Vector};
use graphics::vertex::Vertex;

use crate::entity::aggregate_entity::AggregateUnit;
use crate::entity::{EntityInner, FromToAggregateEntity, Mass};
use crate::modelling::cubic::SimpleCubic;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{BoundingBox, Collision, SimpleVertex, Volume};

#[derive(Debug)]
pub struct SimpleEntity<T> {
    pub model: SimpleCubic,
    pub pose: Pose,
    pub mass: Mass,
    pub(crate) bounding_box: Option<BoundingBox<SimpleVertex, 3, T>>,
    pub velocity: Vector<3>,
}

impl<T> Default for SimpleEntity<T> {
    fn default() -> Self {
        Self {
            mass: Default::default(),
            pose: Default::default(),
            model: Default::default(),
            bounding_box: Default::default(),
            velocity: Default::default(),
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

        Self {
            model,
            mass,
            bounding_box,
            ..Self::default()
        }
    }

    pub fn temp_as_simplecubic_and_anim(&self) -> (&SimpleCubic, Pose) {
        (&self.model, self.pose)
    }
}

impl<T> SimpleEntity<T> {
    pub const fn bounding_box(&self) -> Option<&BoundingBox<SimpleVertex, 3, T>>
    where
        Vector<3>: Volume<3>,
        [(); Vector::<3>::ELEMENT_COUNT]:,
    {
        self.bounding_box.as_ref()
    }

    // pub fn detect_collision<F: Fn(&mut T), O: Fn(&mut T)>(
    // &self,
    // self_alter_meta: F,
    // other: &Self,
    // other_alter_meta: O,
    // ) -> bool {
    // let Some(vec) = (try { BoundingBox::detect_collision(
    // self.bounding_box()?,
    // self.pose,
    // self_alter_meta,
    // other.bounding_box()?,
    // other.pose,
    // other_alter_meta,
    // )} ) else {return false};
    //
    //
    // vec
    // (!vec.is_empty()).then_some(vec)
    // }

    // pub fn elastic_collision_no_rotation<F: Fn(&mut T), O: Fn(&mut T)>(
    // &mut self,
    // self_alter_meta: F,
    // other: &mut Self,
    // other_alter_meta: O,
    // ) -> bool {
    // if let Some(mut vec) = self.detect_collision(&self_alter_meta, other,
    // &other_alter_meta) { TODO: do stuff
    // find the best place to work out the collision.
    // const SMALL_TIMESTEP: f32 = 0.00005;
    // let other_push =
    // Pose::new_from_translation(-other.velocity.scale(SMALL_TIMESTEP));
    // let self_push =
    // Pose::new_from_translation(-self.velocity.scale(SMALL_TIMESTEP));
    //
    // self.pose.push_change(self_push);
    // other.pose.push_change(other_push);
    //
    // while let Some(new_vec) =
    // self.detect_collision(&self_alter_meta, other, &other_alter_meta)
    // {
    // vec = new_vec;
    //
    // self.pose.push_change(self_push);
    // other.pose.push_change(other_push);
    // }
    //
    // let (lhs_manyvec, rhs_manyvec): (Vec<Vec<_>>, Vec<Vec<_>>) =
    // vec.into_iter().unzip();
    //
    // let lhs = lhs_manyvec.into_iter().flatten();
    // let rhs = rhs_manyvec.into_iter().flatten();
    //
    // let lhs_norm = get_average_normal(lhs);
    // let rhs_norm = get_average_normal(rhs);
    //
    // TODO: remove unwrap and test special case of convex shape in box
    // let Some(average) = (lhs_norm.v() - rhs_norm.v()).try_normalize() else {
    // return false;
    // };
    //
    // `average` = direction of force (in which direction?)
    //
    // elastic collision
    // let self_change = 2.0 * other.mass.0 / (other.mass.0 + self.mass.0)
    // average.v().dot(other.velocity - self.velocity);
    //
    // let other_change = 2.0 * self.mass.0 / (other.mass.0 + self.mass.0)
    // average.v().dot(self.velocity - other.velocity);
    //
    // self.velocity += average.v() * self_change;
    // other.velocity += average.v() * other_change;
    //
    // true
    // } else {
    // false
    // }
    // }

    pub fn update(&mut self, timestep: f32) {
        let delta_pose = Pose::new_from_translation(self.velocity.scale(timestep));
        self.pose.push_change(delta_pose);
    }
}

pub fn get_average_normal<I: IntoIterator<Item = [SimpleVertex; 3]>>(input: I) -> UnitVector<3> {
    input
        .into_iter()
        .map(|triangle| {
            triangle
                .into_iter()
                .map(|vertex| vertex.normal.v())
                .sum::<Vector<3>>()
                .normalize()
        })
        .sum::<Vector<3>>()
        .normalize()
}

impl<TThis> Collision for SimpleEntity<TThis> {
    type FuncInputs = <BoundingBox<SimpleVertex, 3, TThis> as Collision>::FuncInputs;
    type Meta = <BoundingBox<SimpleVertex, 3, TThis> as Collision>::Meta;
    type Transform<'a> = <BoundingBox<SimpleVertex, 3, TThis> as Collision>::Transform<'a>;

    fn collides<
        T: Collision,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, T::FuncInputs, &mut T::Meta),
    >(
        &self,
        transform: Self::Transform<'_>,
        target: &T,
        target_transform: T::Transform<'_>,
        alter_meta: F,
    ) -> bool
    where
        for<'a> T::Transform<'a>: Clone,
    {
        let Some(bbox) = &self.bounding_box else {
            return false;
        };
        bbox.collides(transform, target, target_transform, alter_meta)
    }
}

impl<T> FromToAggregateEntity<T> for SimpleEntity<T> {
    type Model = SimpleCubic;

    #[allow(private_interfaces)]
    fn from_aggregate_unit(entity: AggregateUnit<T, Self::Model>, pose: Pose) -> Self {
        let AggregateUnit {
            model,
            bounding_box,
            ..
        } = entity;
        Self {
            model,
            pose,
            mass: Default::default(),
            bounding_box,
            velocity: Default::default(),
        }
    }

    fn to_pair_model_bbox(
        self,
    ) -> (
        Self::Model,
        Option<<Self::Model as EntityInner>::BoundingBox<T>>,
    ) {
        let Self {
            model,
            bounding_box,
            ..
        } = self;
        (model, bounding_box)
    }
}
