use engine::entity::AggregateGroup;
use engine::framebuffer::DefaultFramebuffer;
use engine::modelling::cubic::geometry::Pose;
use engine::modelling::{BloomGroup, QuadGroup, SimpleCubicGroup, SkeletonBbGroup, SkyBoxGroup};
use engine::{EngResult, VecDraw, opengl_shaders};

use crate::state::{State, StateError};

impl State {
    pub fn prep_draw<'a>(
        &'a mut self,
        default_framebuffer: &'a DefaultFramebuffer,
    ) -> EngResult<VecDraw<'a, StateError>, StateError> {
        let time = self.time;

        let mut out = VecDraw::new();

        out.push(SkyBoxGroup::new(
            engine::opengl_shaders::skybox_hdr(),
            &self.hdr_fb,
            &self.skybox,
            &self.camera,
            &(),
            vec![],
        ));

        out.push(AggregateGroup::new(
            opengl_shaders::hdr(),
            &self.hdr_fb,
            &self.camera,
            &(),
            &self.ns_light_group,
            vec![(
                &self.grappling_hook,
                vec![
                    None,
                    Some((&self.grapple_chain_anim, time - self.reset_time)),
                    None,
                ],
            )],
            vec![],
        ));

        // out.push(SimpleCubicGroup::new(
        // opengl_shaders::hdr(),
        // &self.hdr_fb,
        // &self.camera,
        // &(),
        // &self.ns_light_group,
        // vec![(&self.morgan.model, self.morgan.pose, None)],
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
            out.push(BloomGroup::new(default_framebuffer, &self.bloom));
        } else {
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
