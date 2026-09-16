use std::collections::HashSet;
use std::fmt::Debug;
use std::rc::Rc;

use engine::entity::{Mass, Physics, SkeletonEntity};
use engine::geometry::Pose;
use engine::linear_algebra::{UnitVector, Vector};
use engine::modelling::bounding_box::no_alter_meta;
use engine::modelling::cubic::material::Material;
use engine::modelling::cubic::{CalculatedAnimation, SimpleCubic};
use engine::modelling::{Collision, CubicVertex, SkeletonCubic};
use engine::{ColourRGBA, Key, SimpleEntity};
use utils::hash_matches;

use crate::camera::GameCamera;

pub struct Player {
    pub entity: SkeletonEntity<()>,
    // velocity: Vector<3>,
}

impl Player {
    pub fn new() -> Self {
        const PLAYER_SIZE: f32 = 0.1;
        let mut player = SimpleEntity::from_simple_cubic(
            SimpleCubic::cube(PLAYER_SIZE, Rc::new(Material::white())).build(),
            0,
            Mass::new_inverse(1.0),
            |_| (),
        );
        player.physics.pose =
            Pose::new_from_translation(Vector::new([0.25, 0.5 + PLAYER_SIZE * 0.5, 0.25]));
        player.physics.velocity = Vector::ZERO;
        Player {
            entity: player.into(),
        }
    }

    pub fn root(&self) -> Pose {
        self.entity.physics.pose
    }

    pub fn can_see(&mut self, can_see_player: bool) {
        if can_see_player {
            self.entity
                .model
                .temp_set_all_material(Rc::new(Material::from_colour(ColourRGBA::RED)));
        } else {
            self.entity
                .model
                .temp_set_all_material(Rc::new(Material::from_colour(ColourRGBA::GREEN)));
        }
    }

    // pub fn get_pair_model_bind_pose(&self) -> (&SkeletonCubic,
    // CalculatedAnimation) { self.entity.get_pair_model_bind_pose()
    // }

    pub fn get_pair_model(&self) -> (&SkeletonCubic, CalculatedAnimation) {
        self.entity.debug_get_pair_model_bind_pose()
    }

    pub fn controls(
        &mut self,
        camera: &GameCamera,
        keyboard: &HashSet<Key>,
        self_normal: Option<UnitVector<3>>,
    ) -> Vector<3> {
        // let on_box = self
        // .entity
        // .collides(
        // animation,
        // &(),
        // environment,
        // env_anim,
        // &(),
        // no_alter_meta::<SkeletonEntity<()>, E>,
        // )
        // .is_some();
        // let (_, on_box) = self
        // .entity
        // .move_collide_repel(animation, environment, env_anim);
        // let (_, on_ground) = self
        // .entity
        // .move_collide_repel(animation, ground, ground_anim);

        const UP: UnitVector<3> = UnitVector::Y;
        // gravity
        let mut force_accum = Vector::ZERO;

        let grounded = self_normal.is_some();
        //.map(|unit| unit.v().dot(UP.v()) > 1.0 / 2.0_f32.sqrt())
        //.unwrap_or(false);

        let camera_pose = &camera.camera.pose;

        if grounded {
            hash_matches! {keyboard:
                &Key::W => force_accum += camera_pose.motion_forward(None).v(),
                &Key::S => force_accum += -camera_pose.motion_forward(None).v(),
                &Key::A => force_accum += camera_pose.left().v(),
                &Key::D => force_accum += -camera_pose.left().v(),
                &Key::Space => *self.entity.physics.velocity.y_mut() = 1.0,
            }
        }

        force_accum
    }

    pub fn physics<E>(
        &mut self,
        camera: &GameCamera,
        keyboard: &HashSet<Key>,
        animation: &CalculatedAnimation,
        environment: &mut E,
        environment_animation: &E::Transform<'_>,
        frame_time: f32,
    ) where
        E: Physics,
        <E as Collision>::Vertex: CubicVertex + PartialEq,
    {
        let (self_depth_in_other, _) =
            self.entity
                .move_collide_repel(animation, environment, environment_animation);

        let mut force_accum = self.controls(camera, keyboard, self_depth_in_other);

        let mass = self.entity.physics.mass.mass();
        const COEF_FRICTION: f32 = 1.0;
        // gravity
        force_accum += -Vector::Y;

        self.entity.physics.process(
            frame_time,
            |vel, _pos| (Vector::Y * vel.dot(Vector::Y) - vel) * COEF_FRICTION * mass + force_accum,
            0.001,
        );
    }
}
