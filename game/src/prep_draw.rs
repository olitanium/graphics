use engine::Event::Query;
use engine::array_vec::arrvec;
use engine::entity::AggregateGroup;
use engine::framebuffer::DefaultFramebuffer;
use engine::modelling::cubic::geometry::{AggregateCalcAnim, CalculatedAnimation};
use engine::modelling::cubic::lighting::shadow::{PointLightIlluminatesGroup, ShadowListLights};
use engine::modelling::cubic::lighting::simple::ListLights;
use engine::modelling::cubic::{Camera, SkeletonCubicGroup};
use engine::modelling::{BloomGroup, QuadGroup, ShadowGroup, SimpleCubicGroup, SkyBoxGroup};
use engine::query::HasDrawnQuery;
use engine::{PollResult, ShaderProgram, VecDraw, opengl_shaders, shader_program};

use crate::state::{State, StateError};

impl State {
    pub fn draw<'a>(
        &'a mut self,
        default_framebuffer: &'a DefaultFramebuffer,
    ) -> PollResult<'a, StateError> {
        let mut out = VecDraw::new();

        let mut x = if self.toggle_b {
            vec![
                self.origin_cube.get_pair_model_bind_pose(),
                self.player.get_pair_model_bind_pose(),
            ]
        } else {
            vec![]
        };

        if self.toggle_m {
            x.push(self.small_cube.get_pair_model_bind_pose());
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
                x,
                sun,
                vec![],
            )
            .unwrap(),
        );

        // out.push(
        // SkeletonCubicGroup::new(
        // opengl_shaders::hdr_skeleton(),
        // &self.hdr_fb,
        // &self.game_camera.camera,
        // &(),
        // ListLights {
        // point: arrvec![&self.sun.light.light],
        // ..Default::default()
        // },
        // vec![],
        // vec![],
        // )
        // .unwrap(),
        // );
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

        if !self.toggle_g {
            out.push(BloomGroup::new(default_framebuffer, &self.bloom_from_hdrfb));
        } else {
            out.push(QuadGroup::new(
                engine::opengl_shaders::exposure(),
                default_framebuffer,
                vec![&self.quad_from_hdrfb],
                vec![(String::from("exposure"), &self.exposure)],
            ));
        }

        PollResult::Draw(out)
    }
}
