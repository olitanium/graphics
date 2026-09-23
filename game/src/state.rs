use core::time::Duration;
use std::collections::HashSet;

use engine::entity::SkeletonEntity;
use engine::framebuffer::Framebuffer;
use engine::framebuffer::attachments::WithDepth;
use engine::framebuffer::traits::FramebufferInternals as _;
use engine::linear_algebra::Vector;
use engine::modelling::cubic::CameraProjection;
use engine::modelling::cubic::lighting::shadow::ShadowPointLight;
use engine::modelling::{Bloom, SkyBox};
use engine::types::TexDim;
use engine::{ApiState, Draw, Event, PollResult};

use crate::camera::GameCamera;
use crate::player::Player;
use crate::sun::Sun;

pub struct State {
    pub game_camera: GameCamera,

    pub exposure: f32,

    pub toggle_g: bool,
    pub toggle_b: bool,
    pub toggle_h: bool,
    pub toggle_m: bool,

    pub sun: Sun,
    pub sun_duplicate_light: ShadowPointLight,
    pub origin_slope: SkeletonEntity<()>,
    pub small_cube: SkeletonEntity<()>,
    pub player: Player,

    pub skybox: SkyBox,

    pub hdr_fb: Framebuffer<3, WithDepth>,
    pub bloom: Bloom,
}

#[derive(Debug)]
pub struct StateError;

utils::error_boilerplate!(StateError);

impl ApiState for State {
    type DrawOutput<'a>
        = Vec<Box<dyn Draw + 'a>>
    where
        Self: 'a;
    type Error = StateError;

    fn poll<'a>(
        &'a mut self,
        events: Vec<Event<'a>>,
    ) -> PollResult<Self::DrawOutput<'a>, Self::Error> {
        let mut frame_time = Duration::ZERO;
        let mut game_duration = Duration::ZERO;
        let mut keyboard = HashSet::new();
        let mut mouse_delta = Vector::default();
        let mut local_string = None;
        let mut default_fb = None;
        let mut can_see_player = false;

        for event in events {
            match event {
                Event::WindowBuffer(buffer) => default_fb = Some(buffer),
                Event::FrameTime(ft) => frame_time = ft,
                Event::ElapsedTime(at) => game_duration = at,
                Event::WindowResize(_size) => {
                    // self.hdr_fb.resize(size);
                    // self.bloom.resize(size);
                    self.game_camera.camera.projection = CameraProjection::Perspective {
                        fov: const { (90.0_f32).to_radians() },
                        aspect: self.hdr_fb.aspect_ratio(),
                        near: 0.1,
                        far: 100.0,
                    };
                }
                Event::Keyboard { keys, buffer } => {
                    keyboard = keys;
                    local_string = Some(buffer);
                }
                Event::Mouse { delta, .. } => mouse_delta = delta,
                Event::Query(queries) => {
                    if let Some((_, query)) = queries.into_iter().find(|(index, _)| *index == 0) {
                        can_see_player = query.answer();
                    }
                }
                _ => {}
            }
        }

        let mut string = local_string.unwrap();

        self.controls(
            mouse_delta,
            &keyboard,
            &mut string,
            frame_time,
            game_duration,
        )?;

        self.player.can_see(can_see_player);

        if string.ends_with('f') {
            string.pop();
            let sixty = Duration::from_secs_f32(1.0 / 60.0);
            let (_, player_anim) = self.player.get_pair_model();
            let (_, slope_anim) = self.origin_slope.debug_get_pair_model_bind_pose();
            self.physics(sixty, &keyboard, game_duration, &player_anim, &slope_anim);
        }

        self.draw(default_fb.unwrap())
    }

    fn new(initial_size: (TexDim, TexDim)) -> Result<Self, Self::Error> {
        let output = Self::new_internal(initial_size)?;
        Ok(output)
    }
}
