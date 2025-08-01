use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::{FramebufferInternals, FramebufferWithoutExtra};
<<<<<<< HEAD
use graphics::shader_program::{ShaderProgram, ShaderProgramContext};
=======
use graphics::shader_program::{ShaderProgram, ShaderProgramContext, Uniform};
>>>>>>> master
use graphics::{Draw, Result};

use super::Quad;

#[derive(Debug)]
pub struct Group<'a, const N: usize, const OUT: usize, D: FramebufferInternals<OUT>> {
    shader: &'a ShaderProgram<Quad<N>, OUT, D::Tex>,
    framebuffer: &'a D,

    quads: Vec<&'a Quad<N>>,
<<<<<<< HEAD
=======
    uniforms: Vec<(String, &'a dyn Uniform)>,
>>>>>>> master
}

impl<'a, const N: usize, const OUT: usize, X: FramebufferWithoutExtra<OUT>> Group<'a, N, OUT, X> {
    pub fn new(
        shader: &'a ShaderProgram<Quad<N>, OUT, X::Tex>,
        framebuffer: &'a X,

        quads: Vec<&'a Quad<N>>,
<<<<<<< HEAD
=======
        uniforms: Vec<(String, &'a dyn Uniform)>,
>>>>>>> master
    ) -> Box<Self> {
        Box::new(Self {
            shader,
            framebuffer,
            quads,
<<<<<<< HEAD
=======
            uniforms,
>>>>>>> master
        })
    }
}

impl<'a, const N: usize, const OUT: usize, D: FramebufferWithoutExtra<OUT>> Draw
    for Group<'a, N, OUT, D>
{
    fn draw(
        self: Box<Self>,
<<<<<<< HEAD
        register: &mut FramebufferContext,
        marker: &mut ShaderProgramContext,
    ) -> Result<()> {
        let mut active_framebuffer = self.framebuffer.bind(register);

        for quad in self.quads {
            let shader = self.shader.use_program(marker);
            quad.draw(shader, &mut active_framebuffer)?;
=======
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<()> {
        let mut active_framebuffer = self.framebuffer.bind(fb_context);
        let mut shader = self.shader.use_program(sp_context);
        shader.set_uniform_dyn(self.uniforms);

        for quad in self.quads {
            quad.draw(&mut shader, &mut active_framebuffer)?;
>>>>>>> master
        }

        Ok(())
    }
}
