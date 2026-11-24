use std::collections::HashSet;

use engine::entity::{AggregateEntity, AggregateModelIndex};
use engine::framebuffer::Framebuffer;
use engine::framebuffer::attachments::WithDepth;
use engine::framebuffer::traits::FramebufferInternals;
use engine::linear_algebra::Vector;
use engine::modelling::cubic::lighting::simple::PointLight;
use engine::modelling::cubic::{
    AggregateCalcAnim,
    Camera,
    CameraProjection,
    SkeletalAnimation,
    SkinningCubic,
};
use engine::modelling::{Bloom, SkyBox};
use engine::types::TexDim;
use engine::{ApiState, Event, PollResult, VecDraw};

pub struct State {
    pub camera: Camera,

    pub exposure: f32,
    pub sensitivity: f32,

    pub speed: [f32; 3],

    pub point_light: PointLight,

    pub skybox: SkyBox,
    pub hdr_fb: Framebuffer<3, WithDepth>,

    pub bloom: Bloom,
    pub tail: AggregateEntity<SkinningCubic, ()>,
    pub tail_animation: SkeletalAnimation,
    pub other_tail_index: AggregateModelIndex,
    pub most_recent_animation: Option<AggregateCalcAnim>,
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
        let mut default_fb = None;
        let mut actual_time = 0.0;
        for event in events {
            match event {
                Event::WindowBuffer(buffer) => default_fb = Some(buffer),
                Event::FrameTime(ft) => frame_time = ft.as_secs_f32(),
                Event::ElapsedTime(at) => actual_time = at.as_secs_f32(),
                Event::WindowResize(size) => {
                    eprintln!("yes {size:?}");
                    self.hdr_fb.resize(size);

                    eprintln!("hdrfb: {:?}", self.hdr_fb.size());

                    self.bloom.resize(size);
                    self.camera.projection = CameraProjection::Perspective {
                        fov: (90.0_f32).to_radians(),
                        aspect: self.hdr_fb.aspect_ratio(),
                        near: 0.1,
                        far: 100.0,
                    };
                }
                Event::Keyboard { keys, .. } => {
                    keyboard = keys;
                }
                Event::Mouse { delta, .. } => {
                    mouse_delta = delta;
                }
                _ => {}
            }
        }

        self.controls(mouse_delta, keyboard, frame_time)?;

        self.most_recent_animation = Some(self.physics(actual_time));

        self.draw(default_fb.unwrap())
    }

    fn new(initial_size: (TexDim, TexDim)) -> Result<Self, Self::Error> {
        Self::new_internal(initial_size)
    }
}
