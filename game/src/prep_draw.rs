use engine::framebuffer::DefaultFramebuffer;
use engine::modelling::cubic::SkeletonCubicGroup;
use engine::modelling::cubic::geometry::Pose;
use engine::modelling::{
    BloomGroup,
    BoundingBoxGroup,
    QuadGroup,
    SimpleCubicGroup,
    SkeletonBbGroup,
    SkyBoxGroup,
};
use engine::query::HasDrawnQuery;
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
            (&self.imported.model, self.imported.pose, None, time),
            // (
            // &self.imported_other.model,
            // self.imported_other.pose,
            // None,
            // time,
            // ),
        ];

        let cube = vec![(
            &self.imported_other.model,
            self.imported_other.pose,
            None,
            time,
        )];

        out.push(SkyBoxGroup::new(
            engine::opengl_shaders::skybox_hdr(),
            &self.hdr_fb,
            &self.skybox,
            &self.camera,
            &(),
            vec![],
        ));

        if self.show_bbs {
            // out.push(SkeletonBbGroup::new(
            // &self.hdr_fb,
            // self.simpletail.bounding_box().unwrap(),
            // &self.simpletail.model.skeleton,
            // Pose::IDENTITY,
            // Some(&self.skeletal_animation),
            // time,
            // &self.camera,
            // &(),
            // 3,
            // |_| ColourRGBA::RED,
            // ));
            // out.push(BoundingBoxGroup::new(
            // &self.hdr_fb,
            // self.imported_other.bounding_box().unwrap(),
            // self.imported_other.pose,
            // &self.camera,
            // &(),
            // self.bounding_box_level,
            // |_| ColourRGBA::RED,
            // ));
            out.push(BoundingBoxGroup::new(
                &self.hdr_fb,
                self.imported.bounding_box().unwrap(),
                self.imported.pose,
                &self.camera,
                &(),
                self.bounding_box_level,
                |_| ColourRGBA::BLUE,
            ));
        }

        if !self.draw_models {
            out.push(SkeletonCubicGroup::new(
                opengl_shaders::skeletal_quat(),
                &self.hdr_fb,
                &self.camera,
                &(),
                &self.ns_light_group,
                vec![(
                    &self.simpletail.model,
                    Pose::IDENTITY,
                    Some(&self.skeletal_animation),
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
                cube,
                vec![],
            ));
            let cubgroup = SimpleCubicGroup::new(
                opengl_shaders::hdr(),
                &self.hdr_fb,
                &self.camera,
                &(),
                &self.ns_light_group,
                all_models,
                vec![],
            );
            out.push(HasDrawnQuery::new(0, cubgroup))
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
