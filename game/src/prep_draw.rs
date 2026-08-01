use engine::array_vec::arrvec;
use engine::framebuffer::DefaultFramebuffer;
use engine::modelling::cubic::lighting::shadow::{PointLightIlluminatesGroup, ShadowListLights};
use engine::modelling::{BloomGroup, FramebufferQuad, ScreenQuadGroup, ShadowGroup, SkyBoxGroup};
use engine::query::HasDrawnQuery;
use engine::{PollResult, VecDraw};

use crate::state::{State, StateError};

impl State {
    pub fn draw<'b: 'a, 'a>(
        &'b mut self,
        default_framebuffer: &'b DefaultFramebuffer,
    ) -> PollResult<'a, StateError> {
        let mut out = VecDraw::new();

        let mut opaque = vec![
            self.origin_cube.get_pair_model_bind_pose(),
            self.player.get_pair_model_bind_pose(),
        ];
        if self.toggle_m {
            opaque.push(self.small_cube.get_pair_model_bind_pose());
        }

        let sun = vec![self.sun.inner.get_pair_model_bind_pose()];

        out.push(
            ShadowGroup::new(
                &self.hdr_fb,
                &self.game_camera.camera,
                &(),
                ShadowListLights {
                    point: arrvec![&self.sun.light,],
                    ..Default::default()
                },
                opaque.clone(),
                sun,
                vec![],
            )
            .unwrap(),
        );

        let mut opaque = vec![self.origin_cube.get_pair_model_bind_pose()];
        if self.toggle_m {
            opaque.push(self.small_cube.get_pair_model_bind_pose());
        }
        let illuminates = PointLightIlluminatesGroup::new_dyn(&self.sun_duplicate_light, opaque);
        out.push(illuminates);

        let player_illuminates = PointLightIlluminatesGroup::new_dyn(
            &self.sun_duplicate_light,
            vec![self.player.get_pair_model_bind_pose()],
        );
        out.push(HasDrawnQuery::new(0, player_illuminates));

        out.push(SkyBoxGroup::new(
            engine::opengl_shaders::skybox_hdr(),
            &self.hdr_fb,
            &self.skybox,
            &self.game_camera.camera,
            &(),
            vec![],
        ));

        // out.push(BoundingBoxGroup::new(
        // &self.hdr_fb,
        // self.sun.inner.bounding_box().unwrap().debug_get_first(),
        // self.sun.inner.pose,
        // &self.game_camera.camera,
        // &(),
        // 20,
        // || ColourRGBA::grayscale(1.0, 0.5),
        // ));

        if !self.toggle_g {
            out.push(BloomGroup::new(
                &self.hdr_fb,
                default_framebuffer,
                &self.bloom,
            ));
        } else {
            out.push(ScreenQuadGroup::new(
                engine::opengl_shaders::exposure3() as &'a _,
                default_framebuffer,
                vec![FramebufferQuad::new_from_framebuffer(&self.hdr_fb)],
                vec![(String::from("exposure"), &self.exposure)],
            ));
        }
        PollResult::Draw(Box::new(out))
    }
}
