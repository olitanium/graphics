use std::fmt::Debug;
use std::mem;

use geometry::Pose;
use graphics::linear_algebra::UnitVector;
use triangle_intersection::Vector;

use crate::entity::Mass;
use crate::modelling::bounding_box::no_alter_meta;
use crate::modelling::{Collision, CubicVertex};

mod average_normal;

#[derive(Debug, Default, Clone)]
pub struct SimplePhysics {
    pub pose: Pose,
    pub velocity: Vector<3>,
    pub mass: Mass,
    collision_push_direction: Vec<NormalMax>,
}

impl SimplePhysics {
    pub(crate) fn append_to_collision_dir(&mut self, normal_max: NormalMax) {
        self.collision_push_direction.push(normal_max);
    }

    pub(crate) fn get_clear_collision_dir(&mut self) -> Vec<NormalMax> {
        mem::take(&mut self.collision_push_direction)
    }

    pub fn process<F>(&mut self, timestep: f32, mut external_force: F, nudge: f32) -> &Pose
    where
        F: FnMut(
            Vector<3>, // velocity
            Vector<3>, // position
        ) -> Vector<3>,
    {
        let force = external_force(self.velocity, self.pose.translation());

        let acceleration = force * self.mass.inverse_mass();

        self.pose
            .push_change(Pose::new_from_translation(self.velocity * timestep));

        self.velocity += acceleration * timestep;

        let push_dir = self.get_clear_collision_dir();

        for NormalMax { normal, max_depth } in push_dir {
            let number = max_depth.div_euclid(nudge);
            if number > 0.0 {
                self.pose.push_change(Pose::new_from_translation(
                    -normal.v().scale(nudge * number),
                ));
                let nudge_dir = -f32::max(self.velocity.dot(normal.v()), 0.0) * normal.v();

                self.velocity += nudge_dir;
            }
        }

        &self.pose
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NormalMax {
    pub(crate) normal: UnitVector<3>,
    pub(crate) max_depth: f32,
}

pub trait Physics: for<'a> Collision<VertexTransform<'a> = ()> {
    fn physics(&mut self) -> Option<&mut SimplePhysics>;

    fn move_collide_repel<B>(
        &mut self,
        first_transform: &Self::Transform<'_>,
        other: &mut B,
        other_transform: &B::Transform<'_>,
    ) -> (Option<UnitVector<3>>, Option<UnitVector<3>>)
    where
        B: Physics + ?Sized,
        <Self as Collision>::Vertex: CubicVertex + PartialEq,
        <B as Collision>::Vertex: CubicVertex + PartialEq,
    {
        let Some(collides) = self.collides(
            first_transform,
            &(),
            other,
            other_transform,
            &(),
            no_alter_meta::<Self, B>,
        ) else {
            return (None, None);
        };

        let (mut first_tri, mut other_tri): (Vec<_>, Vec<_>) = collides.into_iter().unzip();

        fn get_average_normal<V: CubicVertex + PartialEq>(
            mut this: &mut [[V; 3]],
        ) -> Option<UnitVector<3>> {
            this.sort_by_cached_key(|pos1| pos1.each_ref().map(|vert| vert.position()));
            this = this.partition_dedup().0;

            let total_normals = this
                .iter()
                .map(|tri| V::triangle_normal(tri).v())
                .sum::<Vector<_>>();

            total_normals.try_normalize()
        }

        fn get_average_normal_and_depth_into_other<V: CubicVertex + PartialEq, V2: CubicVertex>(
            this: &mut [[V; 3]],
            other: &[[V2; 3]],
        ) -> Option<NormalMax> {
            let normal = get_average_normal(this)?;

            let this_depth = this
                .as_flattened()
                .iter()
                .map(|vec| vec.position().dot(normal.v()))
                .max_by(f32::total_cmp)?;

            let other_depth = other
                .as_flattened()
                .iter()
                .map(|vec| vec.position().dot(normal.v()))
                .min_by(f32::total_cmp)?;

            let max_depth = (this_depth - other_depth).abs();

            Some(NormalMax { normal, max_depth })
        }

        fn make_opt<A: ?Sized, B: ?Sized>(
            this: &mut A,
            this_tri: &mut [[<A as Collision>::Vertex; 3]],
            other_tri: &[[B; 3]],
        ) -> Option<UnitVector<3>>
        where
            A: Physics,
            <A as Collision>::Vertex: CubicVertex + PartialEq,
            B: CubicVertex + PartialEq,
        {
            // would use Option::map_or_else but closures both mut borrow
            // `this_tri`.
            match this.physics() {
                None => get_average_normal(this_tri),
                Some(this_phys) => get_average_normal_and_depth_into_other(this_tri, other_tri)
                    .map(|normal_max| {
                        this_phys.append_to_collision_dir(normal_max);
                        normal_max.normal
                    }),
            }
        }

        (
            make_opt(self, &mut first_tri, &other_tri),
            make_opt(other, &mut other_tri, &first_tri),
        )
    }
}
