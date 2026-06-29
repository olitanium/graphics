use std::collections::HashSet;
use std::rc::Rc;

use engine::entity::{CalculateAnimationError, Mass, SkeletonEntity};
use engine::linear_algebra::Vector;
use engine::modelling::cubic::SimpleCubic;
use engine::modelling::cubic::geometry::{CalculatedAnimation, Pose};
use engine::modelling::cubic::material::Material;
use engine::modelling::{Collision, SkeletonCubic};
use engine::{ColourRGBA, Key, SimpleEntity};

pub struct Player {
    entity: SkeletonEntity<()>,
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
            test_key_and_collision(self, environment, keyboard, Key::W, [0.0, 0.0, -frame_time]);
            test_key_and_collision(self, environment, keyboard, Key::S, [0.0, 0.0, frame_time]);
            test_key_and_collision(self, environment, keyboard, Key::A, [-frame_time, 0.0, 0.0]);
            test_key_and_collision(self, environment, keyboard, Key::D, [frame_time, 0.0, 0.0]);
        } else {
            self.entity
                .pose
                .push_change(Pose::new_from_translation(Vector::new([
                    0.0,
                    -frame_time,
                    0.0,
                ])));
        }
    }
}

fn test_key_and_collision(
    player: &mut Player,
    environment: &SkeletonEntity<()>,
    keyboard: &HashSet<Key>,
    key: Key,
    change: [f32; 3],
) {
    if keyboard.contains(&key) {
        let old_pose = player.entity.pose;

        player
            .entity
            .pose
            .push_change(Pose::new_from_translation(Vector::new(change)));

        if simple_test_collision(player, environment) {
            player.entity.pose = old_pose;
        }
    }
}

fn simple_test_collision(player: &Player, environment: &SkeletonEntity<()>) -> bool {
    let (_, anim) = player.entity.get_pair_model_bind_pose();
    let (_, env_anim) = environment.get_pair_model_bind_pose();

    player.entity.collides(
        (&player.entity.model.skeleton, &anim),
        environment,
        (&environment.model.skeleton, &env_anim),
        |_, _, _, _| (),
    )
}
