use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::{FramebufferInternals, FramebufferWithoutExtra};
use graphics::shader_program::{ShaderProgram, ShaderProgramContext};
use graphics::{Draw, Result};

use super::Quad;

#[derive(Debug)]
pub struct Group<'a, const N: usize, const OUT: usize, D: FramebufferInternals<OUT>> {
    shader: &'a ShaderProgram<Quad<N>, OUT, D::Tex>,
    framebuffer: &'a D,

    quads: Vec<&'a Quad<N>>,
}

impl<'a, const N: usize, const OUT: usize, X: FramebufferWithoutExtra<OUT>> Group<'a, N, OUT, X> {
    pub fn new(
        shader: &'a ShaderProgram<Quad<N>, OUT, X::Tex>,
        framebuffer: &'a X,

        quads: Vec<&'a Quad<N>>,
    ) -> Box<Self> {
        Box::new(Self {
            shader,
            framebuffer,
            quads,
        })
    }
}

impl<'a, const N: usize, const OUT: usize, D: FramebufferWithoutExtra<OUT>> Draw
    for Group<'a, N, OUT, D>
{
    fn draw(
        self: Box<Self>,
        register: &mut FramebufferContext,
        marker: &mut ShaderProgramContext,
    ) -> Result<()> {
        let mut active_framebuffer = self.framebuffer.bind(register);

        for quad in self.quads {
            let shader = self.shader.use_program(marker);
            quad.draw(shader, &mut active_framebuffer)?;
        }

        Ok(())
    }
}
