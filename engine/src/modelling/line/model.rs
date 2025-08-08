use graphics::ActiveShaderProgram;
use graphics::colour::ColourRGB;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithoutExtra;
use graphics::linear_algebra::Vector;
use graphics::shader_program::Uniform;
use graphics::vertex_array::VertexArray;

use crate::Result;

struct Line<const N: usize> {
    vertices: Vec<Vector<N>>,
    dummy_vertex_array: VertexArray<()>,
}

impl Line<2> {
    pub(crate) fn draw<'a, const OUT: usize, I: FramebufferWithoutExtra<OUT>>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, Self, I::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, I>,
    ) -> Result<()> {
        active_shader.set_uniform("colour".into(), ColourRGB::WHITE);
        active_shader.set_uniform_vec("position".into(), &self.vertices);

        self.dummy_vertex_array
            .draw(active_shader, active_framebuffer)
    }
}
