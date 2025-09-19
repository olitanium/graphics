use graphics::Draw;
use graphics::framebuffer::traits::FramebufferWithoutExtra;
use graphics::linear_algebra::Matrix;
use graphics::shader_program::Uniform;
use graphics::texture::FlatTexture;

use crate::modelling::TestLineVertex;
use crate::modelling::cubic::Camera;
use crate::modelling::cubic::geometry::CameraPose;
use crate::modelling::line::Line;
use crate::{EngResult, opengl_shaders};

pub struct Group<'a, U, const OUT: usize, F: FramebufferWithoutExtra<OUT>> {
    destination: &'a F,
    line: &'a Line<U>,
    look_at: Matrix<4, 4>,
}

impl<'a, U: Uniform, const OUT: usize, F: FramebufferWithoutExtra<OUT>> Group<'a, U, OUT, F> {
    pub fn new<P: CameraPose>(
        framebuffer: &'a F,
        line: &'a Line<U>,
        camera: &Camera<P>,
        hint: &P::Hint,
    ) -> Box<Self> {
        Box::new(Self {
            destination: framebuffer,
            line,
            look_at: camera.look_at(hint),
        })
    }
}

impl<'a, F: FramebufferWithoutExtra<1, Tex = FlatTexture>> Draw
    for Group<'a, TestLineVertex, 1, F>
{
    fn draw(
        self: Box<Self>,
        fb_context: &mut graphics::framebuffer::FramebufferContext,
        shader_context: &mut graphics::ShaderProgramContext,
    ) {
        let mut active_framebuffer = self.destination.bind(fb_context);
        let mut active_shader = opengl_shaders::line_test().use_program(shader_context);

        active_shader.set_uniform("projtimesview".to_string(), &self.look_at);

        let _: EngResult<(), !> = self.line.draw(&mut active_shader, &mut active_framebuffer);
    }
}
