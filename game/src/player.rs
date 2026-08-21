use std::collections::HashSet;
use std::rc::Rc;

use engine::entity::Mass;
use engine::geometry::Pose;
use engine::linear_algebra::{UnitVector, Vector};
use engine::modelling::cubic::SimpleCubic;
use engine::modelling::cubic::material::Material;
use engine::{ColourRGBA, Key, SimpleEntity};
use utils::hash_matches;

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
        player.physics.pose =
            Pose::new_from_translation(Vector::new([0.25, PLAYER_SIZE * 0.5, 0.25]));
        player.physics.velocity = Vector::ZERO;
        Player {
            entity: player,
            // velocity: Vector::ZERO,
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

    pub fn get_pair_model(&self) -> (&SimpleCubic, Pose) {
        self.entity.temp_as_simplecubic_and_anim()
    }

    pub fn controls(
        &mut self,
        keyboard: &HashSet<Key>,
        frame_time: f32,
        environment: &mut SimpleEntity<()>,
        ground: &mut SimpleEntity<()>,
    ) {
        let (_, on_box) = self.entity.move_collide_repel(environment, ());
        let (_, on_ground) = self.entity.move_collide_repel(ground, ());

        const UP: UnitVector<3> = UnitVector::Y;
        // gravity
        let mut force_accum = -UP.v();

        let grounded = on_ground
            .map(|unit| unit.v().dot(UP.v()) > 1.0 / 2.0_f32.sqrt())
            .or_else(|| on_box.map(|unit| unit.v().dot(UP.v()) > 1.0 / 2.0_f32.sqrt()))
            .unwrap_or(false);

        if grounded {
            hash_matches! {keyboard:
                &Key::W => force_accum += -Vector::Z,
                &Key::S => force_accum += Vector::Z,
                &Key::A => force_accum += -Vector::X,
                &Key::D => force_accum += Vector::X,
                &Key::Space => *self.entity.physics.velocity.y_mut() = 1.0,
                //&Key::Space => {force_accum += 30.0 * Vector::Y; eprintln!("jump");},
            }
        }

        let mass = self.entity.physics.mass.mass();
        const COEF_FRICTION: f32 = 1.0;
        self.entity.physics.process(
            frame_time,
            |vel, _pos| (Vector::Y * vel.dot(Vector::Y) - vel) * COEF_FRICTION * mass + force_accum,
            0.001,
        );
    }
}
