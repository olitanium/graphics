use graphics::types::VertexBufferLen;
use graphics::{vertex_array, ActiveShaderProgram};
use graphics::colour::ColourRGB;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithoutExtra;
use graphics::linear_algebra::Vector;
use graphics::shader_program::Uniform;

use crate::EngApiResult;


struct Line<const N: usize> {
    vertices: Vec<Vector<N>>,
}

impl<const N: usize> Line<N>
    where Vector<N>: Uniform,
{
    pub(crate) fn draw<'a, NEVER, const OUT: usize, F: FramebufferWithoutExtra<OUT>>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, Self, F::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, F>,
    ) -> EngApiResult<(), NEVER> {
        active_shader.set_uniform("colour".into(), ColourRGB::WHITE);
        active_shader.set_uniform_vec("position".into(), &self.vertices);

        vertex_array::dummy_vertex_array()
            .draw_len(active_shader, active_framebuffer, VertexBufferLen::new(self.vertices.len() as _))
    }
}
