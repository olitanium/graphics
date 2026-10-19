use engine::array_vec::arrvec;
use engine::framebuffer::DefaultFramebuffer;
use engine::modelling::cubic::SimpleCubicDeferredGroup;
use engine::modelling::cubic::lighting::shadow::{PointLightIlluminatesGroup, ShadowListLights};
use engine::modelling::cubic::lighting::simple::ListLights;
use engine::modelling::{BloomGroup, FramebufferQuad, ScreenQuadGroup, ShadowGroup, SkyBoxGroup};
use engine::query::HasDrawnQuery;
use engine::{Draw, PollResult, VecDraw, deferred, opengl_shaders};

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
            self.sun.model_and_pose(),
        ];
        if self.toggle_m {
            opaque.push(self.small_cube.temp_as_simplecubic_and_anim());
        }

        out.push(SimpleCubicDeferredGroup::new(
            &self.hdr_fb,
            opaque,
            &self.game_camera.camera,
            &(),
            vec![],
        ));

        out.push(deferred::SimpleDeferredCompleteGroup::new(
            default_framebuffer,
            &self.hdr_fb,
            ListLights::default(),
        ));

        PollResult::Draw(out)
    }
}
