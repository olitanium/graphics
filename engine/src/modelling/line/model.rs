use graphics::types::VertexBufferLen;
use graphics::{vertex_array, ActiveShaderProgram};
use graphics::colour::ColourRGB;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithoutExtra;
use graphics::linear_algebra::Vector;
use graphics::shader_program::Uniform;

use crate::EngResult;


pub struct Line<const N: usize> {
    vertices: Vec<Vector<N>>,
}

impl<const N: usize> Line<N> {
    pub fn new(vertices: Vec<Vector<N>>) -> Self {
        Self {
            vertices,
        }
    }

    pub fn as_mut(&mut self) -> &mut Vec<Vector<N>> {
        &mut self.vertices
    }
}

impl<const N: usize> Line<N>
    where Vector<N>: Uniform,
{
    pub(crate) fn draw<'a, const OUT: usize, F: FramebufferWithoutExtra<OUT>>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, Self, F::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, F>,
    ) -> EngResult<(), !> {
        active_shader.set_uniform("in_colour".into(), [ColourRGB::WHITE, ColourRGB::BLACK, ColourRGB::WHITE, ColourRGB::BLACK].as_slice());
        active_shader.set_uniform("in_position".into(), &self.vertices);

        println!("len = {}", self.vertices.len());
        println!("self = {:?}", self.vertices);

        vertex_array::dummy_vertex_array()
            .draw_len(active_shader, active_framebuffer, VertexBufferLen::new(self.vertices.len() as _))?;

        Ok(())
    }
}