use alloc::ffi::CString;
use core::error::Error;

use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::{AttachmentWithoutExtra, FramebufferWithoutExtra};
use graphics::shader_program::{ShaderProgram, ShaderProgramContext, Uniform};
use graphics::texture::FlatTexture;
use graphics::{Draw, DrawOut};

use super::FramebufferQuad;
use crate::modelling::Quad;
#[derive(Debug)]
pub struct Group<
    'a,
    const N: usize,
    const OUT: usize,
    O,
    D: FramebufferWithoutExtra<OUT>,
    X: AttachmentWithoutExtra<Tex = FlatTexture>,
> {
    shader: &'a ShaderProgram<Quad<N>, O, OUT, D::Tex>,
    framebuffer: &'a D,

    quads: Vec<FramebufferQuad<'a, N, X>>,
    uniforms: Vec<(CString, &'a dyn Uniform)>,
}

impl<
    'a,
    const N: usize,
    const OUT: usize,
    O,
    X: AttachmentWithoutExtra<Tex = FlatTexture>,
    D: FramebufferWithoutExtra<OUT>,
> Group<'a, N, OUT, O, D, X>
{
    pub fn new(
        shader: &'a ShaderProgram<Quad<N>, O, OUT, D::Tex>,
        framebuffer: &'a D,

        quads: Vec<FramebufferQuad<'a, N, X>>,
        uniforms: Vec<(CString, &'a dyn Uniform)>,
    ) -> Box<Self> {
        Box::new(Self {
            shader,
            framebuffer,
            quads,
            uniforms,
        })
    }
}

impl<
    const N: usize,
    const OUT: usize,
    O,
    X: AttachmentWithoutExtra<Tex = FlatTexture>,
    D: FramebufferWithoutExtra<OUT>,
> Draw for Group<'_, N, OUT, O, D, X>
{
    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>> {
        let mut active_framebuffer = self.framebuffer.bind(fb_context)?;
        let mut shader = self.shader.use_program(sp_context);
        shader.set_uniform_dyn(self.uniforms)?;

        let err: Vec<_> = self
            .quads
            .into_iter()
            .filter_map(|quad| quad.draw(&mut shader, &mut active_framebuffer).err())
            .collect();

        err.is_empty().ok_or_else(|| format!("{err:?}"))?;

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
