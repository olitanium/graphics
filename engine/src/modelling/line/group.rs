use graphics::{framebuffer::traits::FramebufferWithoutExtra, linear_algebra::Matrix, shader_program, texture::FlatTexture, Draw, ShaderProgram};

use crate::{modelling::{cubic::{geometry::YieldsPose, Camera}, line::Line}, opengl_shaders, EngResult};

pub struct Group<'a, const N: usize, const OUT: usize, F: FramebufferWithoutExtra<OUT>> {
    destination: &'a F,
    line: &'a Line<N>,
    look_at: Matrix<4,4>,
}

impl<'a, const N: usize, const OUT: usize, F: FramebufferWithoutExtra<OUT>> Group<'a, N, OUT, F> {
    pub fn new<P: YieldsPose>(framebuffer: &'a F,
        line: &'a Line<N>,
        camera: &Camera<P>,
        hint: &P::Hint,
    ) -> Box<Self> {
        Box::new(Self {
            destination: framebuffer,
            line,
            look_at: camera.look_at(hint)
        })
    }
}

impl<'a, F: FramebufferWithoutExtra<1, Tex=FlatTexture>> Draw for Group<'a, 3, 1, F> {
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