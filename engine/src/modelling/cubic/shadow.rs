use std::iter;

use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::{Matrix, Vector};
use graphics::shader_program::Uniform;
use graphics::texture::FlatTexture;
use graphics::{ApiResult, Draw, DrawOut, ShaderProgramContext};

use super::camera::Camera;
use super::lighting::shadow::ShadowListLights;
use crate::modelling::SimpleCubic;
use crate::modelling::cubic::camera::CameraPose;
use crate::modelling::cubic::geometry::{self, CalculatedAnimation, Pose, SkeletalAnimation};
use crate::{EngError, EngResult, opengl_shaders};

pub const SHADOW_SHADER_MAX_LIGHTS: usize = 2;

#[derive(Debug)]
pub struct Group<'a, X: FramebufferWithDepth<2, Tex = FlatTexture>> {
    look_at: Matrix<4, 4>,
    position: Vector<3>,

    list_light: &'a ShadowListLights<2>,

    output_framebuffer: &'a X,

    opaque: Vec<(
        &'a SimpleCubic,
        // Pose,
        // Option<&'a SkeletalAnimation>, // animation
        // f32,                           // time
        CalculatedAnimation,
    )>,
    transparent: Vec<(
        &'a SimpleCubic,
        // Pose,
        // Option<&'a SkeletalAnimation>, // animation
        // f32,                           // time
        CalculatedAnimation,
    )>,

    uniforms: Vec<(String, &'a dyn Uniform)>,
}

impl<'a, X: FramebufferWithDepth<2, Tex = FlatTexture>> Group<'a, X> {
    pub fn new<NEVER, O: CameraPose>(
        output_framebuffer: &'a X,
        camera: &Camera<O>,
        hint: &O::Hint,
        list_light: &'a ShadowListLights<2>,
        opaque: Vec<(
            &'a SimpleCubic,
            Pose,
            Option<&'a SkeletalAnimation>, // animation
            f32,                           // time
        )>,
        transparent: Vec<(
            &'a SimpleCubic,
            Pose,
            Option<&'a SkeletalAnimation>, // animation
            f32,                           // time
        )>,
        uniforms: Vec<(String, &'a dyn Uniform)>,
    ) -> EngResult<Box<Self>, NEVER> {
        Ok(Box::new(Self {
            look_at: camera.look_at(hint),
            position: camera.position(hint),

            list_light,
            output_framebuffer,
            opaque: geometry::process_vec(opaque)?,
            transparent: geometry::process_vec(transparent)?,
            uniforms,
        }))
    }
}

impl<'a, NEVER, X: FramebufferWithDepth<2, Tex = FlatTexture>> Draw<EngError<NEVER>>
    for Group<'a, X>
{
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> ApiResult<DrawOut, EngError<NEVER>> {
        self.list_light.gen_depth(
            sp_context,
            fb_context,
            &self.opaque,
            self.position,
            self.uniforms.clone(),
        )?;

        // At this point, all lights have their framebuffers filled with depth
        // information
        let mut active_shadow_shader = opengl_shaders::shadow().use_program(sp_context);
        // SAFETY: because active_shadow_shader is dropped before the end of this
        // function, the references stored cannot leak
        unsafe {
            self.list_light
                .bind(&mut active_shadow_shader, self.position)?;
        }

        active_shadow_shader.set_uniform("projtimesview".to_string(), &self.look_at)?;
        active_shadow_shader
            .set_uniform("camera_position".to_string(), &self.position.homogeneous())?;
        active_shadow_shader.set_uniform_dyn(self.uniforms)?;

        let mut active_output_framebuffer = self.output_framebuffer.bind(fb_context);
        for (model, calculated_animation) in iter::chain(self.opaque, self.transparent) {
            model.draw(
                &mut active_shadow_shader,
                &mut active_output_framebuffer,
                &calculated_animation, /* pose,
                                        * animation,
                                        * time, */
            )?;
        }

        drop(active_shadow_shader);
        DrawOut::OK
    }
}
