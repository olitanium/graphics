use engine::framebuffer::DefaultFramebuffer;
use engine::modelling::cubic::SkeletonCubicGroup;
use engine::modelling::cubic::geometry::Pose;
use engine::modelling::{BloomGroup, QuadGroup, SimpleCubicGroup, SkeletonBbGroup, SkyBoxGroup};
use engine::{ColourRGBA, EngResult, VecDraw, opengl_shaders};

use crate::state::{State, StateError};

impl State {
    pub fn prep_draw<'a>(
        &'a mut self,
        default_framebuffer: &'a DefaultFramebuffer,
    ) -> EngResult<VecDraw<'a, StateError>, StateError> {
        let time = self.time;

        let mut out = VecDraw::new();

        let all_models = vec![
            (
                &self.imported.model,
                self.imported.pose,
                &self.which_animation,
                time,
            ),
            //(&self.imported_other.model, self.which_animation, time),
        ];

        out.push(SkyBoxGroup::new(
            engine::opengl_shaders::skybox_hdr(),
            &self.hdr_fb,
            &self.skybox,
            &self.camera,
            &(),
            vec![],
        ));

        if self.draw_models {
            out.push(SkeletonCubicGroup::new(
                opengl_shaders::skeletal_quat(),
                &self.hdr_fb,
                &self.camera,
                &(),
                &self.ns_light_group,
                vec![(
                    &self.simpletail.model,
                    Pose::IDENTITY,
                    &self.skeletal_animation,
                    time,
                )],
                vec![],
            ));
        } else {
            out.push(SimpleCubicGroup::new(
                opengl_shaders::hdr(),
                &self.hdr_fb,
                &self.camera,
                &(),
                &self.ns_light_group,
                all_models,
                vec![],
            ))
        }
        if self.show_bbs {
            out.push(SkeletonBbGroup::new(
                &self.hdr_fb,
                self.simpletail.bounding_box().unwrap(),
                &self.simpletail.model.skeleton,
                Pose::IDENTITY,
                &self.skeletal_animation,
                time,
                &self.camera,
                &(),
                3,
                |_| ColourRGBA::RED,
            ));
        }

        if self.do_bloom {
            // println!("bloom_on");
            out.push(BloomGroup::new(default_framebuffer, &self.bloom));
        } else {
            // println!("bloom_off");
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
