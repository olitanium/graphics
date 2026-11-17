use engine::array_vec::arrvec;
use engine::deferred::{MakeAmbientOcclusionGroup, ShadowDeferredCompleteGroup};
use engine::framebuffer::DefaultFramebuffer;
use engine::modelling::BloomGroup;
use engine::modelling::cubic::SimpleCubicDeferredGroup;
use engine::modelling::cubic::lighting::shadow::{ListLightIlluminatesGroup, ShadowListLights};
use engine::{Draw, PollResult, VecDraw};

use crate::state::{State, StateError};

impl State {
    pub fn draw<'a>(
        &'a self,
        default_framebuffer: &'a DefaultFramebuffer,
    ) -> PollResult<Vec<Box<dyn Draw + 'a>>, StateError> {
        let mut out = VecDraw::new();

        let mut opaque = vec![
            self.origin_cube.temp_as_simplecubic_and_anim(),
            self.player.get_pair_model(),
            self.small_cube.temp_as_simplecubic_and_anim(),
        ];

        let lights = ShadowListLights {
            point: arrvec![&self.sun.light],
            ..ShadowListLights::default()
        };
        out.push(ListLightIlluminatesGroup::new_dyn(
            lights.clone(),
            opaque.clone(),
            self.game_camera.camera.position(&()),
        ));

        opaque.push(self.sun.model_and_pose());
        out.push(SimpleCubicDeferredGroup::new(
            &self.gbuffer,
            opaque,
            &self.game_camera.camera,
            &(),
            vec![],
        ));

        out.push(MakeAmbientOcclusionGroup::new(
            &self.gbuffer,
            &self.ambient_occlusion_buffer,
            64,
            &self.game_camera.camera,
            &(),
            self.toggle_m,
        ));

        out.push(ShadowDeferredCompleteGroup::<_, 3>::new(
            &self.hdr_out,
            &self.gbuffer,
            Some(&self.ambient_occlusion_buffer),
            lights,
            &self.game_camera.camera,
            &(),
        ));

        out.push(BloomGroup::new(
            &self.hdr_out,
            default_framebuffer,
            &self.bloom,
        ));

        // out.push(ScreenQuadGroup::new(
        // opengl_shaders::quad3(),
        // default_framebuffer,
        // FramebufferQuad::new_from_framebuffer(&self.hdr_out),
        // vec![],
        // ));

        PollResult::Draw(out)
    }
}
