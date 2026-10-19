use alloc::ffi::CString;
use core::error::Error;

use geometry::Pose;
use graphics::framebuffer::traits::FramebufferWithoutExtra as _;
use graphics::linear_algebra::Matrix;
use graphics::shader_program::Uniform;
use graphics::{Draw, DrawOut, ShaderProgram, framebuffer};

use crate::deferred::GBuffer;
use crate::modelling::cubic::{Camera, CameraPose, SimpleCubic};
use crate::opengl_shaders;

pub struct DeferredGroup<'a> {
    framebuffer: &'a GBuffer,
    opaque: Vec<(&'a SimpleCubic, Pose)>,
    camera_look_at: Matrix<4, 4>,

    uniforms: Vec<(CString, &'a dyn Uniform)>,
}

impl<'a> DeferredGroup<'a> {
    pub fn new<O>(
        framebuffer: &'a GBuffer,
        opaque: Vec<(&'a SimpleCubic, Pose)>,
        camera: &Camera<O>,
        hint: &O::Hint,
        uniforms: Vec<(CString, &'a dyn Uniform)>,
    ) -> Box<Self>
    where
        O: CameraPose,
    {
        Box::new(Self {
            framebuffer,
            opaque,
            camera_look_at: camera.look_at(hint),
            uniforms,
        })
    }
}

impl Draw for DeferredGroup<'_> {
    fn draw_sized(
        self,
        fb_context: &mut framebuffer::FramebufferContext,
        sp_context: &mut graphics::ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut active_shader =
            ShaderProgram::use_program(opengl_shaders::deferred_simple(), sp_context);
        let mut active_framebuffer = self.framebuffer.bind(fb_context)?;

        active_shader.set_uniform(&c"projtimesview", &self.camera_look_at)?;
        active_shader.set_uniform_dyn(self.uniforms)?;

        let err: Vec<_> = self
            .opaque
            .into_iter()
            .filter_map(|(model, pose)| {
                model
                    .draw(&mut active_shader, &mut active_framebuffer, pose)
                    .err()
            })
            .collect();

        err.is_empty().ok_or_else(|| format!("{err:?}"))?;

        DrawOut::OK
    }

    fn draw(
        self: Box<Self>,
        fb_context: &mut framebuffer::FramebufferContext,
        sp_context: &mut graphics::ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>> {
        let inner = *self;
        inner.draw_sized(fb_context, sp_context)
    }
}
