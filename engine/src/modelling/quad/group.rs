use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::FramebufferWithoutExtra;
use graphics::shader_program::{ShaderProgram, ShaderProgramContext, Uniform};
use graphics::{Draw, DrawOut};

use super::Quad;

#[derive(Debug)]
pub struct Group<'a, const N: usize, const OUT: usize, O, D: FramebufferWithoutExtra<OUT>> {
    shader: &'a ShaderProgram<Quad<N>, O, OUT, D::Tex>,
    framebuffer: &'a D,

    quads: Vec<&'a Quad<N>>,
    uniforms: Vec<(String, &'a dyn Uniform)>,
}

impl<'a, const N: usize, const OUT: usize, O, X: FramebufferWithoutExtra<OUT>>
    Group<'a, N, OUT, O, X>
{
    pub fn new(
        shader: &'a ShaderProgram<Quad<N>, O, OUT, X::Tex>,
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

impl<'a, O, const N: usize, const OUT: usize, D: FramebufferWithoutExtra<OUT>> Draw
    for Group<'a, N, OUT, O, D>
{
    fn draw_sized(
        self,
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

    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> DrawOut {
        let inner = *self;
        inner.draw_sized(fb_context, sp_context)
    }
}
