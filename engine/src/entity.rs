use std::collections::btree_map::Entry;

use graphics::linear_algebra::{UnitVector, Vector};
use graphics::vertex::Vertex;

use crate::TransformBy;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{BoundingBox, Cubic, QuadVertex, SimpleVertex, Volume};

#[derive(Debug)]
pub struct Entity<T> {
    pub model: Cubic,
    pub mass: f32,
    bounding_box: Option<BoundingBox<SimpleVertex, 3, T>>,
    pub velocity: Vector<3>,
}

impl<T: Default> Default for Entity<T> {
    fn default() -> Self {
        Self {
            mass: f32::INFINITY,
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
        let vec = self.bounding_box()?.detect_collision(
            self.model.root(),
            self_alter_meta,
            other.bounding_box()?,
            other.model.root(),
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
            // find the best place to work out the collision.
            // const SMALL_TIMESTEP: f32 = 0.00005;
            // let other_push =
            // Pose::new_from_translation(-other.velocity.scale(SMALL_TIMESTEP));
            // let self_push =
            // Pose::new_from_translation(-self.velocity.scale(SMALL_TIMESTEP));
            //
            // self.model.root_mut().push_change(self_push);
            // other.model.root_mut().push_change(other_push);
            //
            // let mut index = 0;
            // while let Some(new_vec) =
            // self.detect_collision(&self_alter_meta, other, &other_alter_meta)
            // {
            // index += 1;
            // vec = new_vec;
            //
            // self.model.root_mut().push_change(self_push);
            // other.model.root_mut().push_change(other_push);
            // }

            eprintln!("len {:?}", vec.len());

            let count_vec: Vec<_> = vec.iter().map(|vec| (vec.0.len(), vec.1.len())).collect();
            // eprintln!("count {}", index);
            eprintln!("vec_lens {:?}", count_vec);

            let (lhs_manyvec, rhs_manyvec): (Vec<Vec<_>>, Vec<Vec<_>>) = vec.into_iter().unzip();

            let lhs = lhs_manyvec.into_iter().flatten();
            let rhs = rhs_manyvec.into_iter().flatten();

            eprintln!("================ LHS =============  (nose)");
            let lhs_norm = get_average_normal(lhs, false);
            eprintln!("================ RHS =============  (back)");
            let rhs_norm = get_average_normal(rhs, false);

            eprintln!("lhs: {lhs_norm:?}");
            eprintln!("rhs: {rhs_norm:?}");

            // TODO: remove unwrap and test special case of convex shape in box
            let Some(average) = (lhs_norm.v() - rhs_norm.v()).try_normalize() else {
                return false;
            };

            eprintln!("average: {average:?}");

            // let original_momentum = self.mass * self.velocity + other.mass *
            // other.velocity; let original_energy = self.

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
        self.model.root_mut().push_change(delta_pose);
    }
}

pub fn get_average_normal<I: IntoIterator<Item = [SimpleVertex; 3]>>(
    input: I,
    suppress: bool,
) -> UnitVector<3> {
    input
        .into_iter()
        .map(|triangle| {
            triangle
                .into_iter()
                .map(|vertex| vertex.normal.v())
                // TODO: remove
                .map(|vec| {
                    if !suppress {
                        eprintln!("get_avg: {vec:?}");
                    }
                    vec
                })
                // END TODO
                .sum::<Vector<3>>()
                .normalize()
                .v()
        })
        .sum::<Vector<3>>()
        .normalize()
}
