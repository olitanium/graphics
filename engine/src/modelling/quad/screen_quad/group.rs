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
    uniforms: Vec<(String, &'a dyn Uniform)>,
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
        uniforms: Vec<(String, &'a dyn Uniform)>,
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
    'a,
    const N: usize,
    const OUT: usize,
    O,
    X: AttachmentWithoutExtra<Tex = FlatTexture>,
    D: FramebufferWithoutExtra<OUT>,
> Draw for Group<'a, N, OUT, O, D, X>
{
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> DrawOut {
        let mut active_framebuffer = self.framebuffer.bind(fb_context);
        let mut shader = self.shader.use_program(sp_context);
        shader.set_uniform_dyn(self.uniforms).unwrap();

        for quad in self.quads {
            quad.draw(&mut shader, &mut active_framebuffer).unwrap();
        }

        DrawOut::OK
    }
}
