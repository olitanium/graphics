use engine::array_vec::arrvec;
use engine::entity::AggregateGroup;
use engine::framebuffer::DefaultFramebuffer;
use engine::modelling::cubic::AggregateCalcAnim;
use engine::modelling::cubic::lighting::simple::ListLights;
use engine::modelling::{
    BloomGroup,
    FramebufferQuad,
    ScreenQuadGroup,
    SimpleCubicGroup,
    SkyBoxGroup,
};
use engine::{PollResult, VecDraw, opengl_shaders};

use crate::state::{State, StateError};

impl State {
    pub fn draw<'a>(
        &'a mut self,
        default_framebuffer: &'a DefaultFramebuffer,
        grapple_animation: AggregateCalcAnim,
    ) -> PollResult<VecDraw<'a>, StateError> {
        let _time = self.time;

        let mut out = VecDraw::new();

        out.push(SkyBoxGroup::new(
            engine::opengl_shaders::skybox_hdr(),
            &self.hdr_fb,
            &self.skybox,
            &self.camera,
            &(),
            vec![],
        ));

        out.push(
            AggregateGroup::new(
                opengl_shaders::hdr_skeleton(),
                &self.hdr_fb,
                &self.camera,
                &(),
                ListLights {
                    point: arrvec![&self.ns_point_light],
                    ..Default::default()
                },
                vec![(&self.grappling_hook.entity, grapple_animation)],
                vec![],
            )
            .unwrap(),
        );

        if let Some(bear) = &self.bear {
            out.push(SimpleCubicGroup::new(
                opengl_shaders::hdr_simple(),
                &self.hdr_fb,
                &self.camera,
                &(),
                ListLights {
                    point: arrvec![&self.ns_point_light],
                    ..Default::default()
                },
                vec![bear.inner.temp_as_simplecubic_and_anim()],
                vec![],
            ))
        }

        // out.push(SimpleCubicGroup::new(
        // opengl_shaders::hdr(),
        // &self.hdr_fb,
        // &self.camera,
        // &(),
        // &self.ns_light_group,
        // vec![(&self.bear.model, self.bear.pose, None)],
        // Vec::new(),
        // )?);
        //
        // out.push(SimpleCubicGroup::new(
        // opengl_shaders::hdr(),
        // &self.hdr_fb,
        // &self.camera,
        // &(),
        // &self.ns_light_group,
        // vec![(&self.origin_cube.model, Pose::IDENTITY, None)],
        // Vec::new(),
        // )?);

        if self.toggle_g {
            out.push(BloomGroup::new(
                &self.hdr_fb,
                default_framebuffer,
                &self.bloom,
            ));
        } else {
            let fb = FramebufferQuad::new_from_framebuffer(&self.hdr_fb);
            out.push(ScreenQuadGroup::new(
                engine::opengl_shaders::exposure3(),
                default_framebuffer,
                vec![fb],
                vec![(String::from("exposure"), &self.exposure)],
            ));
        }

        PollResult::Draw(out)
    }
}
