use core::num;

use engine::framebuffer::DefaultFramebuffer;
use engine::linear_algebra::Vector;
use engine::modelling::{
    BloomGroup,
    BoundingBoxGroup,
    QuadGroup,
    ShadowGroup,
    SkeletonBbGroup,
    SkyBoxGroup,
};
use engine::{ColourRGBA, EngResult, TransformBy, VecDraw, entity};

use crate::state::{State, StateError};

impl State {
    pub fn prep_draw<'a>(
        &'a mut self,
        default_framebuffer: &'a DefaultFramebuffer,
    ) -> EngResult<VecDraw<'a>, StateError> {
        let time = self.time;

        let mut out = VecDraw::new();

        let all_models = vec![
            (&self.imported.model, self.imported.pose, &self.which_animation, time),
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

        if self.show_bbs {
            if let Some(bb) = self.imported.bounding_box() {
                out.push(SkeletonBbGroup::new(
                    &self.hdr_fb,
                    bb,
                    &self.imported.model.skeleton,
                    self.imported.pose,
                    &self.which_animation,
                    time,
                    &self.camera,
                    &(),
                    self.bounding_box_level,
                    |(has_collided, vec)| {
                        ColourRGBA::BLUE
                    },
                ));
            }
            /*if let Some(bb) = self.imported_other.bounding_box() {
                out.push(BoundingBoxGroup::new(
                    &self.hdr_fb,
                    bb,
                    &self.imported_other.model.skeleton,
                    &self.camera,
                    &(),
                    self.bounding_box_level,
                    |(collided, vec)| {
                        // let mut array = entity::get_average_normal(vec.into_iter().map(|triangle|
                        // triangle.transform_by(&self.imported_other.model.root())),
                        // true).v().into_inner();
                        let num_triangles = vec.len() * 3;
                        let transform = self.imported_other.model.root();
                        let array = vec
                            .into_iter()
                            .flat_map(move |triangle| {
                                triangle
                                    .transform_by(&transform)
                                    .into_iter()
                                    .map(|vertex| vertex.normal.v())
                            })
                            .sum::<Vector<3>>()
                            .normalize()
                            .v()
                            .into_inner();

                        if array[2] < 0.0 {
                            if *collided {
                                ColourRGBA::grayscale(0.2, 1.0)
                            } else {
                                ColourRGBA::grayscale(0.0, 1.0)
                            }
                        } else {
                            if *collided {
                                eprintln!("ERROR DETECTED");
                                ColourRGBA::RED
                            } else {
                                ColourRGBA::grayscale(1.0, 1.0)
                            }
                        }
                        // ColourRGBA::new_from_arr_alpha(array, 1.0)
                    },
                ));
            }*/
        }

        if self.draw_models {
            out.push(ShadowGroup::new(
                &self.hdr_fb,
                &self.camera,
                &(),
                &self.light_group,
                all_models,
                vec![],
                vec![],
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
