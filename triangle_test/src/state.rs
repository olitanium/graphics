use std::rc::Rc;

use engine::entity::Mass;
use engine::linear_algebra::{UnitVector, Vector};
use engine::modelling::cubic::camera::ManualPose;
use engine::modelling::cubic::geometry::Pose;
use engine::modelling::cubic::lighting::simple::ListLights;
use engine::modelling::cubic::material::Material;
use engine::modelling::cubic::{Camera, MeshBuilder, SimpleCubic, camera};
use engine::modelling::{Collision, IncompleteVertex, SimpleCubicGroup};
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

pub struct State {
    triangle_one: SimpleEntity<()>,
    triangle_two: SimpleEntity<()>,

    camera: Camera,
}

impl ApiState for State {
    type DrawOutput<'a> = Box<dyn Draw + 'a>;
    type Error = ();

    fn new(
        initial_size: (engine::types::TexDim, engine::types::TexDim),
    ) -> Result<Self, Self::Error> {
        let aspect = TexDim::aspect_ratio(initial_size.0, initial_size.1);
        let camera = camera::builder()
            .perspective(90f32.to_radians(), aspect, 0.1, 10.0)
            .pose(ManualPose::new_fixed_forward_up(
                Vector::new([0.0, 0.0, 2.0]),
                -UnitVector::Z,
                UnitVector::Y,
            ))
            .build();

        let triangle_one = {
            let triangle_one_vertices = [-Vector::X + Vector::Y + Vector::Z, -Vector::X, Vector::Y]
                .map(|vec| IncompleteVertex::new(vec).texture(vec.truncate()));

            let mut triangle_one_cubic =
                SimpleCubic::builder().cull_face(shader_program::CullFace::DoNotCull);
            let mut mesh_builder = MeshBuilder::new().material(Rc::new(Material::white()));

            mesh_builder.push_triangle(triangle_one_vertices);
            triangle_one_cubic.push_mesh(mesh_builder);

            let triangle_one_model = triangle_one_cubic.build();

            SimpleEntity::from_simple_cubic(triangle_one_model, 1, Mass(1.0), |_| ())
        };

        let triangle_two = {
            let triangle_two_vertices = [Vector::ZERO, Vector::X, -Vector::Y]
                .map(|vec| IncompleteVertex::new(vec).texture(vec.truncate()));

            let mut triangle_two_cubic =
                SimpleCubic::builder().cull_face(shader_program::CullFace::DoNotCull);
            let mut mesh_builder = MeshBuilder::new().material(Rc::new(Material::black()));

            mesh_builder.push_triangle(triangle_two_vertices);
            triangle_two_cubic.push_mesh(mesh_builder);

            let triangle_one_model = triangle_two_cubic.build();

            SimpleEntity::from_simple_cubic(triangle_one_model, 1, Mass(1.0), |_| ())
        };

        Ok(Self {
            camera,
            triangle_one,
            triangle_two,
        })
    }

    fn poll<'a>(
        &'a mut self,
        events: Vec<engine::Event<'a>>,
    ) -> engine::PollResult<Self::DrawOutput<'a>, Self::Error> {
        let mut opt_default = None;
        let mut frame_time = Default::default();
        let mut keys = Default::default();

        for event in events {
            match event {
                Event::FrameTime(in_ft) => frame_time = in_ft,
                Event::WindowBuffer(in_buff) => opt_default = Some(in_buff),
                Event::Keyboard { keys: in_keys, .. } => keys = in_keys,
                _ => {}
            }
        }

        let frame_time = frame_time.as_secs_f32();

        fn move_triangle_two(state: &mut State, translation: [f32; 3]) {
            state
                .triangle_two
                .pose
                .push_change(Pose::new_from_translation(Vector::new(translation)));
        }

        for key in keys {
            match key {
                Key::Escape => return PollResult::Close,
                Key::W => move_triangle_two(self, [0.0, frame_time, 0.0]),
                Key::S => move_triangle_two(self, [0.0, -frame_time, 0.0]),
                Key::A => move_triangle_two(self, [-frame_time, 0.0, 0.0]),
                Key::D => move_triangle_two(self, [frame_time, 0.0, 0.0]),
                Key::Space => move_triangle_two(self, [0.0, 0.0, frame_time]),
                Key::LeftShift => move_triangle_two(self, [0.0, 0.0, -frame_time]),
                _ => {}
            }
        }

        let test_collision =
            self.triangle_one
                .collides((), &self.triangle_two, (), |_, _, _, _| ());

        if test_collision.is_some() {
            self.triangle_one
                .model
                .temp_set_all_material(Rc::new(Material::from_colour(ColourRGBA::GREEN)));
            eprintln!("yes");
        } else {
            self.triangle_one
                .model
                .temp_set_all_material(Rc::new(Material::from_colour(ColourRGBA::RED)));
            eprintln!("no");
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
