use alloc::ffi::CString;
use core::error::Error;
use core::iter;

use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::{Matrix, Vector};
use graphics::shader_program::Uniform;
use graphics::texture::FlatTexture;
use graphics::{Draw, DrawOut, ShaderProgramContext};

use super::camera::Camera;
use super::lighting::shadow::ShadowListLights;
use crate::modelling::SkeletonCubic;
use crate::modelling::cubic::animation::CalculatedAnimation;
use crate::modelling::cubic::camera::CameraPose;
use crate::opengl_shaders;

pub const SHADOW_SHADER_MAX_LIGHTS: usize = 2;

#[derive(Debug)]
pub struct Group<'a, X: FramebufferWithDepth<3, Tex = FlatTexture>> {
    look_at: Matrix<4, 4>,
    position: Vector<3>,

    list_light: ShadowListLights<'a, 2>,

    output_framebuffer: &'a X,

    opaque: Vec<(
        &'a SkeletonCubic,
        // Pose,
        // Option<&'a SkeletalAnimation>, // animation
        // f32,                           // time
        CalculatedAnimation,
    )>,
    transparent: Vec<(
        &'a SkeletonCubic,
        // Pose,
        // Option<&'a SkeletalAnimation>, // animation
        // f32,                           // time
        CalculatedAnimation,
    )>,

    uniforms: Vec<(CString, &'a dyn Uniform)>,
}

impl<'a, X: FramebufferWithDepth<3, Tex = FlatTexture>> Group<'a, X> {
    pub fn new<O>(
        output_framebuffer: &'a X,
        camera: &Camera<O>,
        hint: &O::Hint,
        list_light: ShadowListLights<'a, 2>,
        opaque: Vec<(&'a SkeletonCubic, CalculatedAnimation)>,
        transparent: Vec<(&'a SkeletonCubic, CalculatedAnimation)>,
        uniforms: Vec<(CString, &'a dyn Uniform)>,
    ) -> Box<Self>
    where
        O: CameraPose,
    {
        Box::new(Self {
            look_at: camera.look_at(hint),
            position: camera.position(hint),

            list_light,
            output_framebuffer,
            opaque,
            transparent,
            uniforms,
        })
    }
}

impl<X: FramebufferWithDepth<3, Tex = FlatTexture>> Draw for Group<'_, X> {
    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>> {
        self.list_light
            .gen_depth(
                sp_context,
                fb_context,
                &self.opaque,
                self.position,
                &self.uniforms,
            )
            .map_err(|err| err.to_string())?;

        // At this point, all lights have their framebuffers filled with depth
        // information
        let mut active_shadow_shader = opengl_shaders::shadow().use_program(sp_context);
        // SAFETY: because active_shadow_shader is dropped before the end of
        // this function, the references stored cannot leak
        unsafe {
            self.list_light
                .bind(&mut active_shadow_shader, self.position)?;
        }

        active_shadow_shader.set_uniform(&c"projtimesview", &self.look_at)?;
        active_shadow_shader.set_uniform(&c"camera_position", &self.position.homogeneous())?;
        active_shadow_shader.set_uniform_dyn(self.uniforms)?;

        let mut active_output_framebuffer = self.output_framebuffer.bind(fb_context)?;

        let mut errs: Vec<_> = iter::chain(self.opaque, self.transparent)
            .filter_map(|(model, calculated_animation)| {
                model
                    .draw(
                        &mut active_shadow_shader,
                        &mut active_output_framebuffer,
                        &calculated_animation,
                    )
                    .err()
            })
            .collect();

        drop(active_shadow_shader);

        // TODO: can we recover this?
        if let Some(first) = errs.pop() {
            return Err(first.into());
        }

        DrawOut::OK
    }

    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>> {
        let inner = *self;
        inner.draw_sized(fb_context, sp_context)
    }
}
