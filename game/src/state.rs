use std::collections::HashSet;
use std::rc::Rc;

use engine::eng_state::EngState;
use engine::framebuffer::attachments::WithDepth;
use engine::framebuffer::traits::FramebufferInternals;
use engine::framebuffer::{DefaultFramebuffer, Framebuffer};
use engine::linear_algebra::Vector;
use engine::modelling::cubic::Camera;
use engine::modelling::cubic::camera::{ManualPose, Projection};
use engine::modelling::cubic::geometry::Pose;
use engine::modelling::cubic::lighting::shadow::ShadowListLights;
use engine::modelling::cubic::lighting::simple::ListLights;
use engine::modelling::cubic::material::Material;
use engine::modelling::{
    Bloom,
    BloomGroup,
    Cubic,
    Line,
    Quad,
    QuadGroup,
    SHADOW_SHADER_MAX_LIGHTS,
    ShadowGroup,
    SkyBox,
    SkyBoxGroup,
    TestLineVertex,
};
use engine::types::TexDim;
use engine::{ApiState, ColourRGB, ColourRGBA, EngError, EngResult, Entity, Event, VecDraw};

pub struct State {
    pub time: f32,
    pub game_start: Option<f32>,
    pub camera: Camera<ManualPose>,

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

    pub light: Cubic,
    pub containers: Vec<Cubic>,
    pub testbool: bool,
    pub imported: Entity,
    pub imported_other: Entity,
    pub blank: Cubic,

    /*pub imported_bounding_box_model: Cubic,
    pub imported_other_bounding_box_model: Cubic,*/
    pub bounding_box_level: usize,

    pub which_animation: usize,

    // pub player: Cubic,
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
    ) -> EngResult<VecDraw<'a>, StateError> {
        let mut frame_time = 0.0;
        let mut keyboard = HashSet::new();
        let mut mouse_delta = (0.0, 0.0);
        let mut string = String::new();
        let mut local_string = &mut string;
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
                Event::TextBuffer(string) => local_string = string,
                Event::Mouse {
                    buttons: _,
                    position: _,
                    delta,
                } => {
                    mouse_delta = (delta.0 as f32, delta.1 as f32);
                }
                _ => {}
            }
        }

        eprintln!("fps: {}", 1.0 / frame_time);

        self.controls(mouse_delta, keyboard, local_string, frame_time)?;

        self.physics(frame_time);

        let out = self.prep_draw(default_fb.unwrap())?;
        Ok(out)
    }

    fn new(initial_size: (TexDim, TexDim)) -> EngResult<Self, StateError> {
        let output = Self::new_(initial_size)?;
        Ok(output)
    }
}

impl State {
    fn physics(&mut self, frame_time: f32) {
        let game_start = *self.game_start.get_or_insert(self.time);
        let time = self.time - game_start;

        let light_pos = Vector::new([
            5.0 * (time / 10.0).sin(),
            5.0 * (time / 10.0).sin(),
            5.0 * (time / 10.0).cos(),
        ]);

        if let Some(light) = self.light_group.point.get_mut(0) {
            light.light.position = light_pos;
            *self.light.root_mut() = Pose::new_from_translation(light_pos);
        }

        // if !self
        // .imported
        // .elastic_collision_no_rotation(&mut self.imported_other)
        // {}
        self.imported_other.update(frame_time);
        self.imported.update(frame_time);

        if self
            .imported
            .elastic_collision_no_rotation(&mut self.imported_other)
        {
            self.imported.velocity = Vector::ZERO;
            self.imported_other.velocity = Vector::ZERO;
        }

        if let Some(light) = self.light_group.spot.get_mut(0) {
            light.light.direction = self.camera.direction(&());
            light.light.position = self.camera.position(&());
        }

        let bounding_box_level = self.bounding_box_level;

        /*if let Some(bbox) = self.imported.bounding_box() {
            self.imported_bounding_box_model = bbox
                .transform_by_pose(self.imported.model.root())
                .make_model(
                    bounding_box_level,
                    Rc::new(Material::from_colour_alpha(ColourRGB::RED, 0.5)),
                )
                .build();
        }*/

        /*if let Some(bbox) = self.imported_other.bounding_box() {
            self.imported_other_bounding_box_model = bbox
                .transform_by_pose(self.imported_other.model.root())
                .make_model(
                    bounding_box_level,
                    Rc::new(Material::from_colour(ColourRGBA::BLUE)),
                )
                .build();
        }*/
    }

    fn prep_draw<'a>(
        &'a mut self,
        default_framebuffer: &'a DefaultFramebuffer,
    ) -> EngResult<VecDraw<'a>, StateError> {
        let time = self.time;

        let mut out = VecDraw::new();

        let all_models = {
            let mut all_models = vec![
                (&self.imported.model, self.which_animation, time),
                (&self.imported_other.model, self.which_animation, time),
            ];

            /*if self.show_bbs {
                all_models.extend(vec![
                    (&self.imported_bounding_box_model, 0, 0.0),
                    (&self.imported_other_bounding_box_model, 0, 0.0),
                ]);
            }*/
            all_models
        };

        out.push(SkyBoxGroup::new(
            engine::opengl_shaders::skybox_hdr(),
            &self.hdr_fb,
            &self.skybox,
            &self.camera,
            &(),
            vec![],
        ));

        out.push(ShadowGroup::new(
            &self.hdr_fb,
            &self.camera,
            &(),
            &self.light_group,
            all_models,
            vec![],
            vec![],
        ));

        if self.do_bloom {
            // println!("bloom_on");
            out.push(BloomGroup::new(default_framebuffer, &self.bloom));
        } else {
            // println!("bloom_off");
            out.push(QuadGroup::new(
                engine::opengl_shaders::exposure(),
                default_framebuffer,
                vec![&self.quad_to_draw],
                vec![(String::from("exposure"), &self.exposure)],
            ));
        }

        Ok(out)
    }
}
