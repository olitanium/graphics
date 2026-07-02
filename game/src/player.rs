use std::collections::HashSet;
use std::rc::Rc;

use engine::entity::{Mass, SkeletonEntity};
use engine::linear_algebra::Vector;
use engine::modelling::cubic::SimpleCubic;
use engine::modelling::cubic::geometry::{CalculatedAnimation, Pose};
use engine::modelling::cubic::material::Material;
use engine::modelling::{Collision, SkeletonCubic};
use engine::{ColourRGBA, Key, SimpleEntity};

pub struct Player {
    entity: SkeletonEntity<()>,
    velocity: Vector<3>,
}

impl Player {
    pub fn new() -> Self {
        const PLAYER_SIZE: f32 = 0.1;
        let mut player = SimpleEntity::from_simple_cubic(
            SimpleCubic::cube(PLAYER_SIZE, Rc::new(Material::white())).build(),
            0,
            Mass(1.0),
            |_| (),
        );
        player.pose = Pose::new_from_translation(Vector::new([0.25, PLAYER_SIZE * 0.5, 0.25]));
        Player {
            entity: player.into(),
            velocity: Vector::ZERO,
        }
    }

    pub fn root(&self) -> Pose {
        self.entity.pose
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

    pub fn get_pair_model_bind_pose(&self) -> (&SkeletonCubic, CalculatedAnimation) {
        self.entity.get_pair_model_bind_pose()
    }

    pub fn controls(
        &mut self,
        keyboard: &HashSet<Key>,
        frame_time: f32,
        environment: &SkeletonEntity<()>,
        ground: &SkeletonEntity<()>,
    ) {
        if simple_test_collision(self, ground) {
            test_key_and_push(self, keyboard, Key::W, -Vector::Z, frame_time);
            test_key_and_push(self, keyboard, Key::S, Vector::Z, frame_time);
            test_key_and_push(self, keyboard, Key::A, -Vector::X, frame_time);
            test_key_and_push(self, keyboard, Key::D, Vector::X, frame_time);
        }
        process_force(self, -Vector::Y, frame_time);

        for mask in [Vector::X, Vector::Z] {
            force_in_direction(self, environment, frame_time, mask);
        }
        force_in_direction(self, ground, frame_time, Vector::Y);

        // friction
        let friction = Vector::new([
            (-1.0 / self.entity.mass.0) * self.velocity[0],
            0.0,
            (-1.0 / self.entity.mass.0) * self.velocity[2],
        ]);

        process_force(self, friction, frame_time);
    }
}

fn force_in_direction(
    player: &mut Player,
    environment: &SkeletonEntity<()>,
    frame_time: f32,
    mask: Vector<3>,
) {
    let old_pose = player.entity.pose;

    player.entity.pose.push_change(Pose::new_from_translation(
        player.velocity.elem_wise_mul(mask) * frame_time,
    ));

    if simple_test_collision(player, environment) {
        player.entity.pose = old_pose;
        let inverse_mask = mask.map(|val| if val == 0.0 { 1.0 } else { 0.0 });
        player.velocity = player.velocity.elem_wise_mul(inverse_mask);
    }
}

fn test_key_and_push(
    player: &mut Player,
    keyboard: &HashSet<Key>,
    key: Key,
    force: Vector<3>,
    frame_time: f32,
) {
    if keyboard.contains(&key) {
        process_force(player, force, frame_time);
    }
}

fn process_force(player: &mut Player, force: Vector<3>, frame_time: f32) {
    player.velocity += force * (1.0 / player.entity.mass.0) * frame_time;
}

fn simple_test_collision(player: &Player, environment: &SkeletonEntity<()>) -> bool {
    let (_, anim) = player.entity.get_pair_model_bind_pose();
    let (_, env_anim) = environment.get_pair_model_bind_pose();

    player
        .entity
        .collides(&anim, environment, &env_anim, |_, _, _, _| ())
}
