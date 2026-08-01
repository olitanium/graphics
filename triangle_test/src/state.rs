use std::collections::HashSet;
use std::rc::Rc;

use engine::entity::Mass;
use engine::framebuffer::DefaultFramebuffer;
use engine::linear_algebra::{UnitVector, Vector};
use engine::modelling::cubic::camera::{CameraPose, ManualPose};
use engine::modelling::cubic::lighting::simple::{ListLights, SimpleBindError};
use engine::modelling::cubic::material::Material;
use engine::modelling::cubic::{Camera, CameraBuilder, Mesh, MeshBuilder, SimpleCubic, camera};
use engine::modelling::{IncompleteVertex, SimpleCubicGroup};
use engine::types::TexDim;
use engine::{
    ApiState,
    DrawNothing,
    Event,
    PollResult,
    SimpleEntity,
    modelling,
    opengl_shaders,
    shader_program,
};

pub struct State {
    triangle_one: SimpleEntity<()>,

    camera: Camera,
}

impl ApiState for State {
    type Error = ();

    fn new(
        initial_size: (engine::types::TexDim, engine::types::TexDim),
    ) -> Result<Self, Self::Error> {
        let aspect = TexDim::aspect_ratio(initial_size.0, initial_size.1);
        let camera = camera::builder()
            .perspective(90f32.to_radians(), aspect, 0.1, 10.0)
            .pose(ManualPose::new_fixed_forward_up(
                Vector::new([0.0, 0.0, -2.0]),
                UnitVector::Z,
                UnitVector::Y,
            ))
            .build();

        let triangle_one_vertices = [-Vector::X + Vector::Y, -Vector::X, Vector::Y]
            .map(|vec| IncompleteVertex::new(vec).texture(vec.truncate()));

        let mut triangle_one_cubic =
            SimpleCubic::builder().cull_face(shader_program::CullFace::DoNotCull);
        let mut mesh_builder = MeshBuilder::new().material(Rc::new(Material::white()));

        mesh_builder.push_triangle(triangle_one_vertices);
        triangle_one_cubic.push_mesh(mesh_builder);

        let triangle_one_model = triangle_one_cubic.build();

        let triangle_one =
            SimpleEntity::from_simple_cubic(triangle_one_model, 1, Mass(1.0), |_| ());

        Ok(Self {
            camera,
            triangle_one,
        })
    }

    fn poll<'a>(
        &'a mut self,
        events: Vec<engine::Event<'a>>,
    ) -> engine::PollResult<'a, Self::Error> {
        let mut opt_default = None;
        let mut frame_time = Default::default();
        let mut keys = Default::default();

        for event in events {
            match event {
                Event::FrameTime(in_ft) => frame_time = in_ft,
                Event::WindowBuffer(in_buff) => opt_default = Some(in_buff),
                Event::Keyboard {
                    keys: in_keys,
                    buffer: _,
                } => keys = in_keys,
                _ => {}
            }
        }

        if let Some(default_fb) = opt_default {
            PollResult::Draw(SimpleCubicGroup::new(
                opengl_shaders::hdr_without_bright_simple(),
                default_fb,
                &self.camera,
                &(),
                ListLights::new_ambient(),
                vec![self.triangle_one.temp_as_simplecubic_and_anim()],
                vec![],
            ))
        } else {
            PollResult::Close
        }
    }
}
