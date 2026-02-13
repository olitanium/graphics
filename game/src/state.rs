use std::collections::HashSet;
use std::rc::Rc;

use engine::entity::AggregateEntity;
use engine::framebuffer::Framebuffer;
use engine::framebuffer::attachments::WithDepth;
use engine::framebuffer::traits::FramebufferInternals;
use engine::modelling::cubic::camera::Projection;
use engine::modelling::cubic::geometry::SkeletalAnimation;
use engine::modelling::cubic::lighting::shadow::ShadowListLights;
use engine::modelling::cubic::lighting::simple::ListLights;
use engine::modelling::cubic::material::Material;
use engine::modelling::cubic::{Camera, SkinningCubic};
use engine::modelling::{Bloom, Line, Quad, SHADOW_SHADER_MAX_LIGHTS, SkyBox, TestLineVertex};
use engine::types::TexDim;
use engine::{ApiState, ColourRGBA, Event, PollResult, SimpleEntity};

use crate::controls::ControlOutput;
use crate::new::GrapplingHook;

pub struct State {
    pub reset_time: f32,
    pub time: f32,
    pub game_start: Option<f32>,
    pub camera: Camera,

    pub exposure: f32,
    pub sensitivity: f32,

    pub toggle_g: bool,
    pub toggle_b: bool,

    pub speed: [f32; 3],

    pub line: Line<TestLineVertex>,
    pub angle: f32,
    pub quad_to_draw: Quad<1>,

    pub light_group: ShadowListLights<SHADOW_SHADER_MAX_LIGHTS>,
    pub ns_light_group: ListLights<SHADOW_SHADER_MAX_LIGHTS>,

    pub morgan: Option<SimpleEntity<()>>,
    pub origin_cube: SimpleEntity<()>,
    pub simpletail: SkinningCubic,
    pub aggregate_tail: AggregateEntity<(), SkinningCubic>,
    pub skeletal_animation: SkeletalAnimation,

    pub grappling_hook: GrapplingHook,

    pub bounding_box_level: usize,
    pub toggle_m: bool,

    pub skybox: SkyBox,
    pub hdr_fb: Framebuffer<2, WithDepth>,

    pub bloom: Bloom,
}

#[derive(Debug)]
pub struct StateError;

utils::error_boilerplate!(StateError);

impl ApiState for State {
    type Error = StateError;

    fn poll<'a>(
        &'a mut self,
        events: Vec<Event<'a>>,
        // default_framebuffer: &'a DefaultFramebuffer,
    ) -> PollResult<'a, Self::Error> {
        let mut frame_time = 0.0;
        let mut keyboard = HashSet::new();
        let mut mouse_delta = (0.0, 0.0);
        let mut local_string = None;
        let mut default_fb = None;
        let mut actual_time = 0.0;
        for event in events {
            match event {
                Event::WindowBuffer(buffer) => default_fb = Some(buffer),
                Event::FrameTime(ft) => frame_time = ft as f32,
                Event::ActualTime(at) => actual_time = at as f32,
                Event::WindowResize(size) => {
                    self.hdr_fb.resize(size);
                    // self.bloom.resize(size);
                    self.camera.projection = Projection::Perspective {
                        fov: (90.0_f32).to_radians(),
                        aspect: self.hdr_fb.aspect_ratio(),
                        near: 0.1,
                        far: 100.0,
                    };
                }
                Event::Keyboard(kb) => keyboard = kb,
                Event::TextBuffer(string) => local_string = Some(string),
                Event::Mouse {
                    buttons: _,
                    position: _,
                    delta,
                } => {
                    mouse_delta = (delta.0 as f32, delta.1 as f32);
                }
                Event::Query(queries) => match queries.as_slice() {
                    [(0, query)] if query.get_answer() => {
                        self.origin_cube
                            .model
                            .temp_set_all_material(Rc::new(Material::from_colour(ColourRGBA::RED)));
                    }
                    [(0, query)] if !query.get_answer() => {
                        self.origin_cube.model.temp_set_all_material(Rc::new(
                            Material::from_colour(ColourRGBA::BLUE),
                        ));
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        let mut string = local_string.unwrap();

        let game_start = *self.game_start.get_or_insert(self.time);
        self.time = actual_time - game_start;

        // eprintln!("fps: {}", 1.0 / frame_time);

        match self.controls(mouse_delta, keyboard, &mut string, frame_time, self.time) {
            ControlOutput::Close => return PollResult::Close,
            ControlOutput::Error(x) => return PollResult::Error(x),
            ControlOutput::Data(()) => (),
        }

        let anim = self.physics(frame_time);

        self.draw(default_fb.unwrap(), anim)
    }

    fn new(initial_size: (TexDim, TexDim)) -> Result<Self, Self::Error> {
        let output = Self::new_internal(initial_size)?;
        Ok(output)
    }
}
