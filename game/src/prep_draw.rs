use engine::framebuffer::DefaultFramebuffer;
use engine::modelling::cubic::SkeletonCubicGroup;
use engine::modelling::cubic::geometry::Pose;
use engine::modelling::{
    BloomGroup,
    QuadGroup,
    ShadowGroup,
    SimpleCubicGroup,
    SkeletonBbGroup,
    SkyBoxGroup,
};
use engine::{
    ColourRGBA,
    EngResult,
    ShaderProgram,
    ShaderProgramContext,
    VecDraw,
    opengl_shaders,
    shader_program,
};

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
        // if self.show_bbs {
        // if let Some(bb) = self.imported.bounding_box() {
        // out.push(SkeletonBbGroup::new(
        // &self.hdr_fb,
        // bb,
        // &self.imported.model.skeleton,
        // self.imported.pose,
        // &self.which_animation,
        // time,
        // &self.camera,
        // &(),
        // self.bounding_box_level,
        // |_| {
        // ColourRGBA::BLUE
        // },
        // ));
        // }
        // if let Some(bb) = self.imported_other.bounding_box() {
        // out.push(BoundingBoxGroup::new(
        // &self.hdr_fb,
        // bb,
        // &self.imported_other.model.skeleton,
        // &self.camera,
        // &(),
        // self.bounding_box_level,
        // |(collided, vec)| {
        // let mut array = entity::get_average_normal(vec.into_iter().map(|triangle|
        // triangle.transform_by(&self.imported_other.model.root())),
        // true).v().into_inner();
        // let num_triangles = vec.len() * 3;
        // let transform = self.imported_other.model.root();
        // let array = vec
        // .into_iter()
        // .flat_map(move |triangle| {
        // triangle
        // .transform_by(&transform)
        // .into_iter()
        // .map(|vertex| vertex.normal.v())
        // })
        // .sum::<Vector<3>>()
        // .normalize()
        // .v()
        // .into_inner();
        //
        // if array[2] < 0.0 {
        // if *collided {
        // ColourRGBA::grayscale(0.2, 1.0)
        // } else {
        // ColourRGBA::grayscale(0.0, 1.0)
        // }
        // } else {
        // if *collided {
        // eprintln!("ERROR DETECTED");
        // ColourRGBA::RED
        // } else {
        // ColourRGBA::grayscale(1.0, 1.0)
        // }
        // }
        // ColourRGBA::new_from_arr_alpha(array, 1.0)
        // },
        // ));
        // }*/
        // }

        if self.draw_models {
            // out.push(ShadowGroup::new(
            // &self.hdr_fb,
            // &self.camera,
            // &(),
            // &self.light_group,
            // all_models,
            // vec![],
            // vec![],
            // ));

            out.push(SkeletonCubicGroup::new(
                opengl_shaders::skeletal(),
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
            eprintln!("mat");
        } else {
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
            eprintln!("quat");
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
