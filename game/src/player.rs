use std::collections::HashSet;
use std::rc::Rc;

use engine::entity::{Mass, SkeletonEntity};
use engine::geometry::Pose;
use engine::linear_algebra::Vector;
use engine::modelling::bounding_box::no_alter_meta;
use engine::modelling::cubic::material::Material;
use engine::modelling::cubic::{CalculatedAnimation, SimpleCubic};
use engine::modelling::{Collision, SkeletonCubic};
use engine::{ColourRGBA, Key, SimpleEntity};

pub struct Player {
    entity: SimpleEntity<()>,
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
        player.physics.set_pose_velocity(
            Pose::new_from_translation(Vector::new([0.25, PLAYER_SIZE * 0.5, 0.25])),
            Vector::ZERO,
        );
        Player {
            entity: player,
            // velocity: Vector::ZERO,
        }
    }

    pub fn root(&self) -> Pose {
        *self.entity.physics.pose()
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

    pub fn get_pair_model(&self) -> (&SimpleCubic, Pose) {
        self.entity.temp_as_simplecubic_and_anim()
    }

    pub fn controls(
        &mut self,
        keyboard: &HashSet<Key>,
        frame_time: f32,
        environment: &SimpleEntity<()>,
        ground: &SimpleEntity<()>,
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

        let [velx, _, velz] = self.entity.physics.velocity().into_inner();
        // friction
        let friction = Vector::new([
            -self.entity.physics.mass().inverse_mass() * velx,
            0.0,
            -self.entity.physics.mass().inverse_mass() * velz,
        ]);

        process_force(self, friction, frame_time);
    }
}

fn force_in_direction(
    player: &mut Player,
    environment: &SimpleEntity<()>,
    frame_time: f32,
    mask: Vector<3>,
) {
    let old_pose = *player.entity.physics.pose();
    let old_velocity = *player.entity.physics.velocity();

    let mut new_pose = old_pose;

    new_pose.push_change(Pose::new_from_translation(
        old_velocity.elem_wise_mul(mask) * frame_time,
    ));

    player
        .entity
        .physics
        .set_pose_velocity(new_pose, old_velocity);

    if simple_test_collision(player, environment) {
        player
            .entity
            .physics
            .set_pose_velocity(old_pose, old_velocity);
        let inverse_mask = mask.map(|val| if val == 0.0 { 1.0 } else { 0.0 });
        let new_velocity = player.entity.physics.velocity().elem_wise_mul(inverse_mask);
        player.entity.physics.set_velocity(new_velocity);
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
    let new_velocity = *player.entity.physics.velocity()
        + force * player.entity.physics.mass().inverse_mass() * frame_time;
    player.entity.physics.set_velocity(new_velocity)
}

// fn skeleton_test_collision(player: &Player, environment: &SkeletonEntity<()>)
// -> bool { let (_, env_anim) = environment.get_pair_model_bind_pose();
//
// player
// .entity
// .collides((), environment, &env_anim, |_, _, _, _| ())
// .is_some()
// }

fn simple_test_collision(player: &Player, environment: &SimpleEntity<()>) -> bool {
    player
        .entity
        .collides(
            (),
            environment,
            (),
            no_alter_meta::<SimpleEntity<()>, SimpleEntity<()>>,
        )
        .is_some()
}
