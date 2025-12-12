use std::collections::HashMap;

use engine::entity::AggregateGroup;
use engine::framebuffer::DefaultFramebuffer;
use engine::modelling::cubic::SkeletonBoneIndex;
use engine::modelling::cubic::geometry::Pose;
use engine::modelling::{BloomGroup, QuadGroup, SkyBoxGroup};
use engine::{PollResult, VecDraw, opengl_shaders};

use crate::state::{State, StateError};

impl State {
    pub fn draw<'a>(
        &'a mut self,
        default_framebuffer: &'a DefaultFramebuffer,
    ) -> PollResult<'a, StateError> {
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

        out.push(
            AggregateGroup::new(
                opengl_shaders::hdr(),
                &self.hdr_fb,
                &self.camera,
                &(),
                &self.ns_light_group,
                vec![(
                    &self.grappling_hook.entity,
                    HashMap::from([(
                        self.grappling_hook.chain,
                        (&self.grappling_hook.chain_animation, time - self.reset_time),
                    )]),
                    if self.toggle_b {
                        Some((
                            self.grappling_hook.hook,
                            SkeletonBoneIndex::SKELETON_ROOT_BONE,
                            Pose::IDENTITY,
                        ))
                    } else {
                        None
                    },
                )],
                vec![],
            )
            .unwrap(),
        );

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

        PollResult::Draw(out)
    }
}
