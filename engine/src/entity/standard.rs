use graphics::linear_algebra::{UnitVector, Vector};
use graphics::vertex::Vertex;

use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{BoundingBox, Cubic, SimpleVertex, Volume};

#[derive(Debug)]
pub struct Entity<T> {
    pub model: Cubic,
    pub pose: Pose,
    pub mass: f32,
    bounding_box: Option<BoundingBox<SimpleVertex, 3, T>>,
    pub velocity: Vector<3>,
}

impl<T: Default> Default for Entity<T> {
    fn default() -> Self {
        Self {
            mass: f32::INFINITY,
            pose: Pose::IDENTITY,
            model: Default::default(),
            bounding_box: Default::default(),
            velocity: Default::default(),
        }
    }
}

impl Entity<(bool, Vec<[SimpleVertex; 3]>)> {
    pub fn from_cubic(model: Cubic, depth: usize, mass: f32) -> Option<Self> {
        let bounding_box = BoundingBox::from_cubic(&model, depth, |input| {
            (
                false,
                input.into_iter().map(|(_, triangle)| *triangle).collect(),
            )
        });

        Some(Self {
            model,
            mass,
            bounding_box,
            ..Self::default()
        })
    }
}

impl<T> Entity<T> {
    pub fn bounding_box(&self) -> Option<&BoundingBox<SimpleVertex, 3, T>>
    where
        Vector<3>: Volume<3>,
        [(); Vector::<3>::ELEMENT_COUNT]:,
    {
        self.bounding_box.as_ref()
    }

    pub fn detect_collision<F: Fn(&mut T), O: Fn(&mut T)>(
        &self,
        self_alter_meta: F,
        other: &Self,
        other_alter_meta: O,
    ) -> Option<Vec<(Vec<[SimpleVertex; 3]>, Vec<[SimpleVertex; 3]>)>> {
        let vec = BoundingBox::detect_collision(
            self.bounding_box()?,
            self.pose,
            self_alter_meta,
            other.bounding_box()?,
            other.pose,
            other_alter_meta,
        );

        (!vec.is_empty()).then_some(vec)
    }

    pub fn elastic_collision_no_rotation<F: Fn(&mut T), O: Fn(&mut T)>(
        &mut self,
        self_alter_meta: F,
        other: &mut Self,
        other_alter_meta: O,
    ) -> bool {
        if let Some(mut vec) = self.detect_collision(&self_alter_meta, other, &other_alter_meta) {
            // TODO: do stuff
            // find the best place to work out the collision.
            const SMALL_TIMESTEP: f32 = 0.00005;
            let other_push = Pose::new_from_translation(-other.velocity.scale(SMALL_TIMESTEP));
            let self_push = Pose::new_from_translation(-self.velocity.scale(SMALL_TIMESTEP));

            self.pose.push_change(self_push);
            other.pose.push_change(other_push);

            while let Some(new_vec) =
                self.detect_collision(&self_alter_meta, other, &other_alter_meta)
            {
                vec = new_vec;

                self.pose.push_change(self_push);
                other.pose.push_change(other_push);
            }

            let (lhs_manyvec, rhs_manyvec): (Vec<Vec<_>>, Vec<Vec<_>>) = vec.into_iter().unzip();

            let lhs = lhs_manyvec.into_iter().flatten();
            let rhs = rhs_manyvec.into_iter().flatten();

            let lhs_norm = get_average_normal(lhs);
            let rhs_norm = get_average_normal(rhs);

            // TODO: remove unwrap and test special case of convex shape in box
            let Some(average) = (lhs_norm.v() - rhs_norm.v()).try_normalize() else {
                return false;
            };

            // elastic collision
            let self_change = 2.0 * other.mass / (other.mass + self.mass)
                * average.v().dot(other.velocity - self.velocity);

            let other_change = 2.0 * self.mass / (other.mass + self.mass)
                * average.v().dot(self.velocity - other.velocity);

            self.velocity = self.velocity + average.v() * self_change;
            other.velocity = other.velocity + average.v() * other_change;

            true
        } else {
            false
        }
    }

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
                .v()
        })
        .sum::<Vector<3>>()
        .normalize()
}
