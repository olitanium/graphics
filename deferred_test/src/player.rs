use alloc::rc::Rc;

use engine::entity::Mass;
use engine::geometry::Pose;
use engine::linear_algebra::{Vector, vector};
use engine::modelling::cubic::SimpleCubic;
use engine::modelling::cubic::material::Material;
use engine::{ColourRGBA, SimpleEntity};

pub struct Player {
    entity: SimpleEntity<()>,
}

impl Player {
    pub fn new() -> Self {
        const PLAYER_SIZE: f32 = 0.1;
        let mut entity = SimpleEntity::from_simple_cubic(
            SimpleCubic::cube(PLAYER_SIZE, Rc::new(Material::white())).build(),
            0,
            Mass::new_inverse(1.0),
            |_| (),
        );
        entity.physics.pose = Pose::new_from_translation(vector![0.25, PLAYER_SIZE * 0.5, 0.25]);
        entity.physics.velocity = Vector::ZERO;
        Self { entity }
    }

    pub const fn root(&self) -> Pose {
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

    pub const fn get_pair_model(&self) -> (&SimpleCubic, Pose) {
        self.entity.temp_as_simplecubic_and_anim()
    }

    // pub fn controls<E>(
    // &mut self,
    // animation: &CalculatedAnimation,
    // camera: &GameCamera,
    // keyboard: &HashSet<Key>,
    // frame_time: f32,
    // environment: &mut E,
    // env_anim: &E::Transform<'_>,
    // ground: &mut E,
    // ground_anim: &E::Transform<'_>,
    // ) where
    // E: Physics,
    // <E as Collision>::Vertex: CubicVertex + PartialEq + Debug, // TODO:
    // remove Debug {
    // let (_, on_box) = self
    // .entity
    // .move_collide_repel(animation, environment, env_anim);
    // let (_, on_ground) = self
    // .entity
    // .move_collide_repel(animation, ground, ground_anim);
    //
    // const UP: UnitVector<3> = UnitVector::Y;
    // gravity
    // let mut force_accum = -UP.v();
    //
    // let grounded = on_ground
    // .map(|unit| unit.v().dot(UP.v()) > 1.0 / 2.0_f32.sqrt())
    // .or_else(|| on_box.map(|unit| unit.v().dot(UP.v()) > 1.0 /
    // 2.0_f32.sqrt())) .unwrap_or(false);
    //
    // let camera_pose = &camera.camera.pose;
    //
    // if grounded {
    // hash_matches! {keyboard:
    // &Key::W => force_accum += camera_pose.motion_forward(None).v(),
    // &Key::S => force_accum += -camera_pose.motion_forward(None).v(),
    // &Key::A => force_accum += camera_pose.left().v(),
    // &Key::D => force_accum += -camera_pose.left().v(),
    // &Key::Space => *self.entity.physics.velocity.y_mut() = 1.0,
    // }
    // }
    //
    // let mass = self.entity.physics.mass.mass();
    // const COEF_FRICTION: f32 = 1.0;
    // self.entity.physics.process(
    // frame_time,
    // |vel, _pos| (Vector::Y * vel.dot(Vector::Y) - vel) * COEF_FRICTION * mass + force_accum,
    // 0.001,
    // );
    // }
}
