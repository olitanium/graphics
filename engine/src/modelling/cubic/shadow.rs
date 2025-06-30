use std::iter;

use super::camera::Camera;
use super::geometry::YieldsPose;
use super::lighting::shadow::ShadowListLights;
use super::model::Cubic;
use crate::buffers::framebuffer::{FramebufferContext, FramebufferWithDepth};
use crate::linear_algebra::{Matrix, Vector};
use crate::modelling::draw::Draw;
use crate::shader_program::{ShaderProgram, ShaderProgramContext};
use crate::texture::FlatTexture;
use crate::Result;

pub const SHADOW_SHADER_MAX_LIGHTS: usize = 2;

#[derive(Debug)]
pub struct Group<'a, X: FramebufferWithDepth<2, Tex = FlatTexture>> {
    look_at: Matrix<4, 4>,
    position: Vector<3>,

    list_light: &'a ShadowListLights<2>,

    output_framebuffer: &'a X,

    opaque: Vec<(&'a Cubic, usize /* animation */, f32 /* time */)>,
    transparent: Vec<(&'a Cubic, usize /* animation */, f32 /* time */)>,
}

impl<'a, X: FramebufferWithDepth<2, Tex = FlatTexture>> Group<'a, X> {
    pub fn new<O: YieldsPose>(
        camera: &Camera<O>,
        hint: O::Hint,
        list_light: &'a ShadowListLights<2>,
        opaque: Vec<(&'a Cubic, usize /* animation */, f32 /* time */)>,
        transparent: Vec<(&'a Cubic, usize /* animation */, f32 /* time */)>,
        output_framebuffer: &'a X,
    ) -> Box<Self>
    where
        O::Hint: Clone,
    {
        Box::new(Self {
            look_at: camera.look_at(hint.clone()),
            position: camera.position(hint),

            list_light,
            output_framebuffer,
            opaque,
            transparent,
        })
    }
}

impl<'a, X: FramebufferWithDepth<2, Tex = FlatTexture>> Draw for Group<'a, X> {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<()> {
        self.list_light
            .gen_depth(sp_context, fb_context, &self.opaque, self.position)?;

        // At this point, all lights have their framebuffers filled with depth
        // information
        let mut active_shadow_shader = ShaderProgram::shadow().use_program(sp_context);
        // SAFETY: because active_shadow_shader is dropped before the end of this
        // function, the references stored cannot leak
        unsafe {
            self.list_light
                .bind(&mut active_shadow_shader, self.position);
        }

        active_shadow_shader.set_uniform("projtimesview".to_string(), self.look_at);
        active_shadow_shader.set_uniform("camera_postion".to_string(), self.position.homogeneous());

        let mut active_output_framebuffer = self.output_framebuffer.bind(fb_context);
        for (model, animation, time) in iter::chain(self.opaque, self.transparent) {
            model.draw(
                &mut active_shadow_shader,
                &mut active_output_framebuffer,
                animation,
                time,
            )?;
        }

        drop(active_shadow_shader);

        Ok(())
    }
}
