use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithoutExtra;
use graphics::shader_program::Uniform;
use graphics::types::ElementArrayLen;
use graphics::vertex::NoVertex;
use graphics::{ActiveShaderProgram, vertex_array};

use crate::EngResult;
use crate::opengl_shaders::LINE_SHADER_MAX_VERTICES;

pub struct Line<U> {
    vertices: Vec<U>,
}

impl<U: Uniform> Line<U> {
    pub fn new(vertices: Vec<U>) -> Self {
        Self { vertices }
    }

    pub fn as_mut(&mut self) -> &mut Vec<U> {
        &mut self.vertices
    }

    pub fn as_ref(&self) -> &Vec<U> {
        &self.vertices
    }
}

impl<U: Uniform> Line<U> {
    pub(crate) fn draw<'a, const OUT: usize, F: FramebufferWithoutExtra<OUT>>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, NoVertex, Self, F::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, F>,
    ) -> EngResult<(), !> {
        active_shader.set_uniform("vertex".to_string(), &self.vertices);

        vertex_array::dummy_vertex_array().draw_len(
            active_shader,
            active_framebuffer,
            ElementArrayLen::new(Ord::min(self.vertices.len(), LINE_SHADER_MAX_VERTICES) as _),
        )?;

        Ok(())
    }
}
