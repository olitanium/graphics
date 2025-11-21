use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::FramebufferWithoutExtra;
use graphics::shader_program::{ShaderProgram, ShaderProgramContext, Uniform};
use graphics::{ApiResult, Draw, DrawOut};

use super::Quad;
use crate::EngError;
use crate::modelling::quad::QuadVertex;

#[derive(Debug)]
pub struct Group<'a, const N: usize, const OUT: usize, D: FramebufferWithoutExtra<OUT>> {
    shader: &'a ShaderProgram<QuadVertex, Quad<N>, OUT, D::Tex>,
    framebuffer: &'a D,

    quads: Vec<&'a Quad<N>>,
    uniforms: Vec<(String, &'a dyn Uniform)>,
}

impl<'a, const N: usize, const OUT: usize, X: FramebufferWithoutExtra<OUT>> Group<'a, N, OUT, X> {
    pub fn new(
        shader: &'a ShaderProgram<QuadVertex, Quad<N>, OUT, X::Tex>,
        framebuffer: &'a X,

        quads: Vec<&'a Quad<N>>,
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

impl<'a, NEVER, const N: usize, const OUT: usize, D: FramebufferWithoutExtra<OUT>>
    Draw<EngError<NEVER>> for Group<'a, N, OUT, D>
{
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> ApiResult<DrawOut, EngError<NEVER>> {
        let mut active_framebuffer = self.framebuffer.bind(fb_context);
        let mut shader = self.shader.use_program(sp_context);
        shader.set_uniform_dyn(self.uniforms)?;

        for quad in self.quads {
            quad.draw(&mut shader, &mut active_framebuffer)?;
        }

        DrawOut::OK
    }
}
