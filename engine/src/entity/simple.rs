use graphics::linear_algebra::{UnitVector, Vector};
use graphics::vertex::Vertex;

use crate::entity::{EntityInner, Mass, ToAggregateEntity};
use crate::modelling::cubic::SimpleCubic;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{BoundingBox, Collision, SimpleVertex, SkeletonCubic, Volume};

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

    pub const fn temp_as_simplecubic_and_anim(&self) -> (&SimpleCubic, Pose) {
        (&self.model, self.pose)
    }
}

impl<T> SimpleEntity<T> {
    pub const fn bounding_box(&self) -> Option<&BoundingBox<SimpleVertex, 3, T>>
    where
        Vector<3>: Volume<3>,
        [(); <Vector<3> as Vertex>::ELEMENT_COUNT]:,
    {
        self.bounding_box.as_ref()
    }

    // pub fn elastic_collision_no_rotation(&mut self, other: &mut Self) -> bool {
    // let Some(mut vec) = self.collides((), other, (), |_, _, _, _| {}) else {
    // return false;
    // };
    //
    // const SMALL_TIMESTEP: f32 = 1.0 / 120.0;
    // let other_push =
    // Pose::new_from_translation(-other.velocity.scale(SMALL_TIMESTEP));
    // let self_push =
    // Pose::new_from_translation(-self.velocity.scale(SMALL_TIMESTEP));
    //
    // self.pose.push_change(self_push);
    // other.pose.push_change(other_push);
    //
    // while let Some(new_vec) = self.collides((), other, (), |_, _, _, _| {}) {
    // vec = new_vec;
    //
    // self.pose.push_change(self_push);
    // other.pose.push_change(other_push);
    // }
    //
    // let (lhs_manyvec, rhs_manyvec): (Vec<&_>, Vec<&_>) = vec.into_iter().unzip();
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
    // let self_change = 2.0 * other.mass.0 / (other.mass.0 + self.mass.0);
    // average.v().dot(other.velocity - self.velocity);
    //
    // let other_change = 2.0 * self.mass.0 / (other.mass.0 + self.mass.0);
    // average.v().dot(self.velocity - other.velocity);
    //
    // self.velocity += average.v() * self_change;
    // other.velocity += average.v() * other_change;
    //
    // todo!()
    // }
}

pub fn get_average_normal<I: IntoIterator<Item = [SimpleVertex; 3]>>(input: I) -> UnitVector<3> {
    // input
    // .into_iter()
    // .map(|triangle| {
    // triangle
    // .into_iter()
    // .map(|vertex| vertex.normal.v())
    // .sum::<Vector<3>>()
    // .normalize()
    // .v()
    // })
    // .sum::<Vector<3>>()
    // .normalize()

    // input
    // .into_iter()
    // .flat_map(|triangle| triangle.into_iter().map(|vertex| vertex.normal.v()))
    // .sum::<Vector<3>>()
    // .normalize();

    input
        .into_iter()
        .flatten()
        .map(|vert| vert.normal.v())
        .sum::<Vector<3>>()
        .normalize()
}

impl<TThis> Collision for SimpleEntity<TThis> {
    type FuncInputs = <BoundingBox<SimpleVertex, 3, TThis> as Collision>::FuncInputs;
    type Meta = <BoundingBox<SimpleVertex, 3, TThis> as Collision>::Meta;
    type Transform<'a> = ();
    type Vertex = <BoundingBox<SimpleVertex, 3, TThis> as Collision>::Vertex;

    fn collides<
        'a,
        'b,
        T: Collision,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, T::FuncInputs, &mut T::Meta),
    >(
        &'a self,
        _: Self::Transform<'_>,
        target: &'b T,
        target_transform: T::Transform<'_>,
        alter_meta: F,
    ) -> Option<Vec<(&'a [Self::Vertex; 3], &'b [T::Vertex; 3])>>
    where
        for<'c> T::Transform<'c>: Clone,
    {
        self.bounding_box
            .as_ref()?
            .collides(self.pose, target, target_transform, alter_meta)
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
