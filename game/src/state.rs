use std::collections::HashSet;

use engine::framebuffer::attachments::WithDepth;
use engine::framebuffer::{DefaultFramebuffer, Framebuffer};
use engine::linear_algebra::Vector;
use engine::modelling::cubic::camera::{CameraPose, Projection};
use engine::modelling::cubic::geometry::{Orientation, Pose};
use engine::modelling::cubic::lighting::shadow::ShadowListLights;
use engine::modelling::cubic::lighting::simple::ListLights;
use engine::modelling::cubic::Camera;
use engine::modelling::{
    Bloom,
    BloomGroup,
    Cubic,
    CubicGroup,
    Quad,
    QuadGroup,
    ShadowGroup,
    SkyBox,
    SkyBoxGroup,
    SHADOW_SHADER_MAX_LIGHTS,
};
use engine::shader_program::ShaderProgram;
use engine::types::TexDim;
use engine::{Draw, Event, GlobalState, Result};
pub struct State {
    pub string: String,
    pub time: f32,
    pub camera: Camera<CameraPose>,

    pub exposure: f32,
    pub sensitivity: f32,

    pub do_bloom: bool,

    pub speed: [f32; 3],

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

impl GlobalState for State {
    fn poll<'a>(
        &'a mut self,
        events: Vec<Event>,
        default_framebuffer: &'a DefaultFramebuffer,
    ) -> Result<Vec<Box<dyn Draw + 'a>>> {
        let mut frame_time = 0.0;
        let mut keyboard = HashSet::new();
        let mut mouse_delta = (0.0, 0.0);
        let mut typing_string = String::new();

        for event in events {
            match event {
                Event::CriticalFault => return Err(engine::Error::Close),
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
                Event::TextBuffer(string) => typing_string = string,
                Event::Mouse {
                    buttons: _,
                    position: _,
                    delta,
                } => {
                    mouse_delta = (delta.0 as f32, delta.1 as f32);
                }
            }
        }

        // println!("framerate: {}", 1.0 / frame_time);

        self.string.push_str(&typing_string);

        self.controls(mouse_delta, keyboard, typing_string, frame_time as f32)?;

        self.physics(frame_time);

        self.prep_draw(default_framebuffer)
    }

    fn new(initial_size: (TexDim, TexDim)) -> Result<Self> {
        Self::new_(initial_size)
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
            light.light.direction = self.camera.direction(());
            light.light.position = self.camera.position(());
        }
    }

    fn prep_draw<'a>(
        &'a mut self,
        default_framebuffer: &'a DefaultFramebuffer,
    ) -> Result<Vec<Box<dyn Draw + 'a>>> {
        let time = self.time;

        let mut out: Vec<Box<dyn Draw>> = Vec::new();

        let all_models = {
            let mut all_models = vec![(&self.imported, self.which_animation, time)];

            // all_models.extend(self.containers.iter().map(|x| (x, time)));

            all_models
        };

        let transparent_models = vec![]; // vec![(&self.light, time)];

        out.push(ShadowGroup::new(
            &self.camera,
            (),
            &self.light_group,
            all_models,
            transparent_models,
            &self.hdr_fb,
        ));

        out.push(SkyBoxGroup::new(
            engine::opengl_shaders::skybox_hdr(),
            &self.hdr_fb,
            &self.skybox,
            &self.camera,
            (),
        ));

        if self.do_bloom {
            // println!("bloom_on");
            out.push(BloomGroup::new(default_framebuffer, &self.bloom));
        } else {
            // println!("bloom_off");
            out.push(QuadGroup::new(
                engine::opengl_shaders::quad(),
                default_framebuffer,
                vec![&self.quad_to_draw],
            ));
        }

        Ok(out)
    }
}
