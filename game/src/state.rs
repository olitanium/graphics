use std::collections::HashSet;

use engine::eng_state::EngState;
use engine::framebuffer::attachments::WithDepth;
use engine::framebuffer::traits::FramebufferInternals;
use engine::framebuffer::{DefaultFramebuffer, Framebuffer};
use engine::linear_algebra::Vector;
use engine::modelling::cubic::Camera;
use engine::modelling::cubic::camera::{CameraPose, Projection};
use engine::modelling::cubic::geometry::{Orientation, Pose};
use engine::modelling::cubic::lighting::shadow::ShadowListLights;
use engine::modelling::cubic::lighting::simple::ListLights;
use engine::modelling::{
    Bloom, BloomGroup, Cubic, Line, LineGroup, Quad, QuadGroup, ShadowGroup, SkyBox, SkyBoxGroup, SHADOW_SHADER_MAX_LIGHTS
};
use engine::types::TexDim;
use engine::{EngApiResult, EngError, EngResult, Event, ApiState, VecDraw};

pub struct State {
    pub time: f32,
    pub camera: Camera<CameraPose>,

    pub exposure: f32,
    pub sensitivity: f32,

    pub do_bloom: bool,

    pub speed: [f32; 3],

    pub line: Line<3>,
    pub angle: f32,
    pub quad_to_draw: Quad<1>,

    pub light_group: ShadowListLights<SHADOW_SHADER_MAX_LIGHTS>,
    pub ns_light_group: ListLights<SHADOW_SHADER_MAX_LIGHTS>,

    pub light: Cubic,
    pub containers: Vec<Cubic>,
    pub imported: Cubic,
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

        println!("framerate: {}", 1.0 / frame_time);
        println!("string: {}", local_string);

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
    fn physics(&mut self, _frame_time: f32) {
        let time = self.time;

        let light_pos = Vector::new([
            5.0 * (time / 10.0).sin(),
            5.0 * (time / 10.0).sin(),
            5.0 * (time / 10.0).cos(),
        ]);

        if let Some(light) = self.light_group.point.get_mut(0) {
            light.light.position = light_pos;
            self.light
                .skeleton
                .set(
                    0,
                    Pose::new_from_orientation_translation(Orientation::default(), light_pos),
                )
                .expect("root node at 0");
        }

        if let Some(light) = self.light_group.spot.get_mut(0) {
            light.light.direction = self.camera.direction(&());
            light.light.position = self.camera.position(&());
        }
    }

    fn prep_draw<'a>(
        &'a mut self,
        default_framebuffer: &'a DefaultFramebuffer,
    ) -> EngResult<VecDraw<'a>, StateError> {
        let time = self.time;

        let mut out: VecDraw = Vec::new();

        let all_models = {
            let mut all_models = vec![(&self.imported, self.which_animation, time)];

            //all_models.extend(self.containers.iter().map(|x| (x, 0, time)));

            all_models
        };

        //let transparent_models = vec![]; // vec![(&self.light, time)];

        /*out.push(ShadowGroup::new(
            &self.camera,
            &(),
            &self.light_group,
            all_models,
            transparent_models,
            &self.hdr_fb,
            vec![],
        ));*/

        out.push(LineGroup::new(
            &self.hdr_fb,
            &self.line,
            &self.camera,
            &(),
        ));

        out.push(SkyBoxGroup::new(
            engine::opengl_shaders::skybox_hdr(),
            &self.hdr_fb,
            &self.skybox,
            &self.camera,
            &(),
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

            // out.push(QuadGroup::new(
            // engine::opengl_shaders::exposure(),
            // other_buffer,
            // vec![&self.quad_to_draw],
            // vec![(String::from("exposure"), &self.exposure)],
            // ));
        }

        Ok(out)
    }
}
