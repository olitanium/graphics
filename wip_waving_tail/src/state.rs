use alloc::rc::Rc;
use std::collections::HashSet;

use engine::framebuffer::Framebuffer;
use engine::framebuffer::attachments::WithDepth;
use engine::framebuffer::traits::FramebufferInternals;
use engine::linear_algebra::Vector;
use engine::modelling::cubic::lighting::shadow::{ShadowPointLight, ShadowSpotLight};
use engine::modelling::cubic::lighting::simple::PointLight;
use engine::modelling::cubic::material::Material;
use engine::modelling::cubic::{AggregateCalcAnim, Camera, CameraProjection};
use engine::modelling::{Bloom, Line, SkyBox, TestLineVertex};
use engine::types::TexDim;
use engine::{ApiState, ColourRGBA, Event, PollResult, SimpleEntity, VecDraw};

use crate::bear::Bear;
use crate::grappling_hook::GrapplingHook;

pub struct State {
    pub time: f32,
    pub game_start: Option<f32>,
    pub camera: Camera,

    pub exposure: f32,
    pub sensitivity: f32,

    pub toggle_g: bool,
    pub toggle_b: bool,
    pub toggle_h: bool,

    pub speed: [f32; 3],

    pub line: Line<TestLineVertex>,
    pub angle: f32,

    pub point_light: ShadowPointLight,
    pub spotlight: ShadowSpotLight,
    pub ns_point_light: PointLight,

    pub bear: Option<Bear>,
    pub origin_cube: SimpleEntity<()>,

    pub grappling_hook: GrapplingHook,
    pub grapple_animation: AggregateCalcAnim,

    pub bounding_box_level: usize,
    pub toggle_m: bool,

    pub skybox: SkyBox,
    pub hdr_fb: Framebuffer<3, WithDepth>,

    pub bloom: Bloom,
}

#[derive(Debug)]
pub struct StateError;

utils::error_boilerplate!(StateError);

impl ApiState for State {
    type DrawOutput<'a>
        = VecDraw<'a>
    where
        Self: 'a;
    type Error = StateError;

    fn poll<'a>(
        &'a mut self,
        events: Vec<Event<'a>>,
        // default_framebuffer: &'a DefaultFramebuffer,
    ) -> PollResult<Self::DrawOutput<'a>, Self::Error> {
        let mut frame_time = 0.0;
        let mut keyboard = HashSet::new();
        let mut mouse_delta = Vector::default();
        let mut local_string = None;
        let mut default_fb = None;
        let mut actual_time = 0.0;
        for event in events {
            match event {
                Event::WindowBuffer(buffer) => default_fb = Some(buffer),
                Event::FrameTime(ft) => frame_time = ft.as_secs_f32(),
                Event::ElapsedTime(at) => actual_time = at.as_secs_f32(),
                Event::WindowResize(size) => {
                    self.hdr_fb.resize(size);
                    // self.bloom.resize(size);
                    self.camera.projection = CameraProjection::Perspective {
                        fov: (90.0_f32).to_radians(),
                        aspect: self.hdr_fb.aspect_ratio(),
                        near: 0.1,
                        far: 100.0,
                    };
                }
                Event::Keyboard { keys, buffer } => {
                    keyboard = keys;
                    local_string = Some(buffer);
                }
                Event::Mouse { delta, .. } => {
                    mouse_delta = delta;
                }
                Event::Query(queries) => match queries.as_slice() {
                    [(0, query)] if query.answer_ref() => {
                        self.origin_cube
                            .model
                            .temp_set_all_material(Rc::new(Material::from_colour(ColourRGBA::RED)));
                    }
                    [(0, query)] if !query.answer_ref() => {
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

        self.controls(mouse_delta, keyboard, &mut string, frame_time, self.time)?;

        self.grapple_animation = self.physics(frame_time);

        self.draw(default_fb.unwrap())
    }

    fn new(initial_size: (TexDim, TexDim)) -> Result<Self, Self::Error> {
        Self::new_internal(initial_size)
    }
}
