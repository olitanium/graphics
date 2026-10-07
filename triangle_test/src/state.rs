use alloc::rc::Rc;
use core::error::Error;
use core::fmt;
use core::fmt::Debug;
use core::time::Duration;
use std::collections::HashSet;

use engine::entity::Mass;
use engine::geometry::{Orientation, Pose};
use engine::linear_algebra::{UnitVector, Vector};
use engine::modelling::cubic::lighting::simple::ListLights;
use engine::modelling::cubic::material::Material;
use engine::modelling::cubic::{
    Camera,
    CameraBuilder,
    ManualPoseFixedUp,
    MeshBuilder,
    SimpleCubic,
};
use engine::modelling::{Collision as _, IncompleteVertex, SimpleCubicGroup};
use engine::types::TexDim;
use engine::{
    ApiState,
    ColourRGBA,
    Draw,
    Event,
    Key,
    PollResult,
    SimpleEntity,
    opengl_shaders,
    shader_program,
};
use utils::hash_matches;

pub struct State {
    triangle_one: SimpleEntity<()>,
    triangle_two: SimpleEntity<()>,

    camera: Camera,
}

#[derive(Debug)]
pub struct StateError;
impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}
impl Error for StateError {}

impl ApiState for State {
    type DrawOutput<'a> = Box<dyn Draw + 'a>;
    type Error = StateError;

    fn new(initial_size: (TexDim, TexDim)) -> Result<Self, Self::Error> {
        let aspect = TexDim::aspect_ratio(initial_size.0, initial_size.1);
        let camera = CameraBuilder::new()
            .perspective(90_f32.to_radians(), aspect, 0.1, 10.0)
            .pose(
                const {
                    ManualPoseFixedUp::new_forward(
                        Vector::new([0.0, 0.0, 2.0]),
                        -UnitVector::Z,
                        UnitVector::Y,
                    )
                    .expect("literals are okay")
                },
            )
            .build();

        fn make_triangle(triangle: [Vector<3>; 3]) -> SimpleEntity<()> {
            let triangle_one_vertices =
                triangle.map(|vec| IncompleteVertex::new(vec).texture(vec.truncate()));

            let mut triangle_one_cubic =
                SimpleCubic::builder().cull_face(shader_program::CullFace::DoNotCull);
            let mut mesh_builder = MeshBuilder::new().material(Rc::new(Material::white()));

            mesh_builder.push_triangle(triangle_one_vertices);
            triangle_one_cubic.push_mesh(mesh_builder);

            let triangle_one_model = triangle_one_cubic.build();

            let out = SimpleEntity::from_simple_cubic(
                triangle_one_model,
                1,
                Mass::new_inverse(1.0),
                |_| (),
            );

            // out.bounding_box = out
            // .bounding_box
            // .take()
            // .map(|value| value.new_test_trianges(|_| ()));
            out
        }

        let triangle_one = make_triangle([-Vector::X + Vector::Y, -Vector::X, Vector::Y]);
        let triangle_two = make_triangle([Vector::ZERO, Vector::X, -Vector::Y]);

        Ok(Self {
            triangle_one,
            triangle_two,
            camera,
        })
    }

    fn poll<'a>(
        &'a mut self,
        events: Vec<engine::Event<'a>>,
    ) -> engine::PollResult<Self::DrawOutput<'a>, Self::Error> {
        let mut opt_default = None;
        let mut frame_time = Duration::default();
        let mut keys = HashSet::default();

        for event in events {
            match event {
                Event::FrameTime(in_ft) => frame_time = in_ft,
                Event::WindowBuffer(in_buff) => opt_default = Some(in_buff),
                Event::Keyboard { keys: in_keys, .. } => keys = in_keys,
                _ => {}
            }
        }

        let frame_time = frame_time.as_secs_f32();

        const fn move_triangle_two(state: &mut State, translation: [f32; 3]) {
            let old_pose = &mut state.triangle_two.physics.pose;
            old_pose.push_change(Pose::new_from_translation(Vector::new(translation)));
        }

        const fn rotate_triangle_two(state: &mut State, rotation: Orientation) {
            let old_pose = state.triangle_two.physics.pose;
            let mut new_pose = Pose::new_from_orientation(rotation);
            new_pose.push_change(old_pose);
            state.triangle_two.physics.pose = new_pose;
        }

        hash_matches! { keys:
            &Key::Escape => return PollResult::Close,
            &Key::W => move_triangle_two(self, [0.0, frame_time, 0.0]),
            &Key::S => move_triangle_two(self, [0.0, -frame_time, 0.0]),
            &Key::A => move_triangle_two(self, [-frame_time, 0.0, 0.0]),
            &Key::D => move_triangle_two(self, [frame_time, 0.0, 0.0]),
            &Key::E => rotate_triangle_two(self, Orientation::new_forward_up(Vector::new([frame_time, 0.0, 1.0]).try_normalize().unwrap_or_default(), UnitVector::Y).unwrap_or_default()),
            &Key::Space => move_triangle_two(self, [0.0, 0.0, frame_time]),
            &Key::LeftShift => move_triangle_two(self, [0.0, 0.0, -frame_time]),
        }

        let test_collision =
            self.triangle_one
                .collides(&(), &(), &self.triangle_two, &(), &(), |(), (), (), ()| {
                    true
                });

        if test_collision.is_some() {
            self.triangle_one
                .model
                .temp_set_all_material(Rc::new(Material::from_colour(ColourRGBA::GREEN)));
        } else {
            self.triangle_one
                .model
                .temp_set_all_material(Rc::new(Material::from_colour(ColourRGBA::RED)));
        }

        if let Some(default_fb) = opt_default {
            PollResult::Draw(SimpleCubicGroup::new(
                opengl_shaders::hdr_without_bright_simple(),
                default_fb,
                &self.camera,
                &(),
                ListLights::new_ambient(),
                vec![
                    self.triangle_one.temp_as_simplecubic_and_anim(),
                    self.triangle_two.temp_as_simplecubic_and_anim(),
                ],
                vec![],
            ))
        } else {
            PollResult::Close
        }
    }
}
