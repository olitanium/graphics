use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::{FramebufferInternals, FramebufferWithoutExtra};
use graphics::shader_program::{ShaderProgram, ShaderProgramContext, Uniform};
use graphics::Draw;

use crate::EngResult;

use super::Quad;

#[derive(Debug)]
pub struct Group<'a, const N: usize, const OUT: usize, D: FramebufferWithoutExtra<OUT>> {
    shader: &'a ShaderProgram<Quad<N>, OUT, D::Tex>,
    framebuffer: &'a D,

    quads: Vec<&'a Quad<N>>,
    uniforms: Vec<(String, &'a dyn Uniform)>,
}

impl<'a, const N: usize, const OUT: usize, X: FramebufferWithoutExtra<OUT>> Group<'a, N, OUT, X> {
    pub fn new(
        shader: &'a ShaderProgram<Quad<N>, OUT, X::Tex>,
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

impl<'a, const N: usize, const OUT: usize, D: FramebufferWithoutExtra<OUT>> Draw
    for Group<'a, N, OUT, D>
{
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) {
        let _: EngResult<(), !> = try {
            let mut active_framebuffer = self.framebuffer.bind(fb_context);
            let mut shader = self.shader.use_program(sp_context);
            shader.set_uniform_dyn(self.uniforms);

            for quad in self.quads {
                quad.draw(&mut shader, &mut active_framebuffer)?;
            }
        };
    }
}
