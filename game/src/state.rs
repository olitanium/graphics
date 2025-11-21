use std::collections::HashSet;
use std::rc::Rc;

use engine::eng_state::EngState;
use engine::entity::{SkeletonEntity, SkinningEntity};
use engine::framebuffer::Framebuffer;
use engine::framebuffer::attachments::WithDepth;
use engine::framebuffer::traits::FramebufferInternals;
use engine::modelling::cubic::Camera;
use engine::modelling::cubic::camera::Projection;
use engine::modelling::cubic::geometry::SkeletalAnimation;
use engine::modelling::cubic::lighting::shadow::ShadowListLights;
use engine::modelling::cubic::lighting::simple::ListLights;
use engine::modelling::cubic::material::Material;
use engine::modelling::{
    Bloom,
    Line,
    Quad,
    SHADOW_SHADER_MAX_LIGHTS,
    SimpleCubic,
    SkyBox,
    TestLineVertex,
};
use engine::types::TexDim;
use engine::{ApiState, ColourRGBA, EngError, EngResult, Entity, Event, VecDraw};

pub struct State {
    pub time: f32,
    pub game_start: Option<f32>,
    pub camera: Camera,

    pub exposure: f32,
    pub sensitivity: f32,

    pub do_bloom: bool,
    pub show_bbs: bool,

    pub speed: [f32; 3],

    pub line: Line<TestLineVertex>,
    pub angle: f32,
    pub quad_to_draw: Quad<1>,

    pub light_group: ShadowListLights<SHADOW_SHADER_MAX_LIGHTS>,
    pub ns_light_group: ListLights<SHADOW_SHADER_MAX_LIGHTS>,

    pub light: SimpleCubic,
    pub containers: Vec<SimpleCubic>,
    pub testbool: bool,
    pub imported: Entity<()>,
    pub imported_other: Entity<()>,
    pub simpletail: SkinningEntity<()>,
    pub skeletal_animation: SkeletalAnimation,

    pub bounding_box_level: usize,
    pub draw_models: bool,

    pub which_animation: SkeletalAnimation,

    pub skybox: SkyBox,
    pub hdr_fb: Framebuffer<2, WithDepth>,

    pub bloom: Bloom,
}

#[derive(Debug, Clone, Copy)]
pub struct StateError;

utils::error_boilerplate!(StateError);
engine::derive_api_state!(State);

impl EngState for State {
    type Error = StateError;

    fn poll<'a>(
        &'a mut self,
        events: Vec<Event<'a>>,
        // default_framebuffer: &'a DefaultFramebuffer,
    ) -> EngResult<VecDraw<'a, StateError>, StateError> {
        let mut frame_time = 0.0;
        let mut keyboard = HashSet::new();
        let mut mouse_delta = (0.0, 0.0);
        let mut local_string = None;
        let mut default_fb = None;
        for event in events {
            match event {
                Event::WindowBuffer(buffer) => default_fb = Some(buffer),
                Event::FrameTime(ft) => frame_time = ft as f32,
                Event::ActualTime(at) => self.time = at as f32,
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
                        self.imported_other
                            .model
                            .temp_set_all_material(Rc::new(Material::from_colour(ColourRGBA::RED)));
                    }
                    [(0, query)] if !query.get_answer() => {
                        self.imported_other.model.temp_set_all_material(Rc::new(
                            Material::from_colour(ColourRGBA::BLUE),
                        ));
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        let mut string = local_string.unwrap();

        // eprintln!("fps: {}", 1.0 / frame_time);

        self.controls(mouse_delta, keyboard, &mut string, frame_time)?;

        self.physics(frame_time);

        let out = self.prep_draw(default_fb.unwrap())?;
        Ok(out)
    }

    fn new(initial_size: (TexDim, TexDim)) -> EngResult<Self, StateError> {
        let output = Self::new_internal(initial_size)?;
        Ok(output)
    }
}
