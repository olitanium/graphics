use engine::array_vec::arrvec;
use engine::entity::AggregateGroup;
use engine::framebuffer::DefaultFramebuffer;
use engine::modelling::cubic::lighting::simple::ListLights;
use engine::modelling::{BloomGroup, FramebufferQuad, ScreenQuadGroup, SkyBoxGroup};
use engine::{PollResult, VecDraw, opengl_shaders};

use crate::state::{State, StateError};

impl State {
    pub fn draw<'a>(
        &'a mut self,
        default_framebuffer: &'a DefaultFramebuffer,
    ) -> PollResult<VecDraw<'a>, StateError> {
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
                opengl_shaders::skeletal_quat(),
                &self.hdr_fb,
                &self.camera,
                &(),
                ListLights {
                    point: arrvec![&self.point_light],
                    ..Default::default()
                },
                vec![(&self.tail, self.most_recent_animation.as_ref().unwrap())],
                vec![],
            )
            .unwrap(),
        );

        if true {
            out.push(BloomGroup::new(
                &self.hdr_fb,
                default_framebuffer,
                &self.bloom,
            ));
        } else {
            out.push(ScreenQuadGroup::new(
                engine::opengl_shaders::exposure3(),
                default_framebuffer,
                FramebufferQuad::new_from_framebuffer(&self.hdr_fb),
                vec![(c"exposure".to_owned(), &self.exposure)],
            ));
        }

        PollResult::Draw(out)
    }
}
