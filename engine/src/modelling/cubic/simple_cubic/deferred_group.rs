use std::error::Error;
use std::ffi::CString;

use geometry::Pose;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Matrix;
use graphics::shader_program::Uniform;
use graphics::texture::FlatTexture;
use graphics::{Draw, DrawOut, ShaderProgram, framebuffer};

use crate::modelling::cubic::{Camera, CameraPose, SimpleCubic};
use crate::opengl_shaders;

const GBUFFER_ELEMENTS: usize = 4;

pub struct DeferredGroup<'a, F>
where
    F: FramebufferWithDepth<GBUFFER_ELEMENTS, Tex = FlatTexture>,
{
    framebuffer: &'a F,
    opaque: Vec<(&'a SimpleCubic, Pose)>,
    camera_look_at: Matrix<4, 4>,

    uniforms: Vec<(CString, &'a dyn Uniform)>,
}

impl<'a, F> DeferredGroup<'a, F>
where
    F: FramebufferWithDepth<GBUFFER_ELEMENTS, Tex = FlatTexture>,
{
    pub fn new<O>(
        framebuffer: &'a F,
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

impl<F> Draw for DeferredGroup<'_, F>
where
    F: FramebufferWithDepth<GBUFFER_ELEMENTS, Tex = FlatTexture>,
{
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
