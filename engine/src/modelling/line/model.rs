use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithoutExtra;
use graphics::shader_program::{SetUniformError, Uniform};
use graphics::types::ElementArrayLen;
use graphics::vertex::NoVertex;
use graphics::vertex_array::VertexArrayDrawError;
use graphics::{ActiveShaderProgram, Model, vertex_array};
use utils::error_boilerplate;

use crate::opengl_shaders::LINE_SHADER_MAX_VERTICES;

#[derive(Debug)]
pub struct Line<U> {
    vertices: Vec<U>,
}

impl<U: Uniform> Line<U> {
    pub const fn new(vertices: Vec<U>) -> Self {
        Self { vertices }
    }

    pub const fn get_mut(&mut self) -> &mut Vec<U> {
        &mut self.vertices
    }

    pub const fn get(&self) -> &Vec<U> {
        &self.vertices
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DrawError<'a> {
    SetUniform(SetUniformError<'a>),
    VertexArrayDraw(VertexArrayDrawError<'a>),
    LineTooLong { len: usize },
}
error_boilerplate!(<'a> DrawError<'a>);

impl<U: Uniform> Line<U> {
    pub(crate) fn draw<'b, 'a, const OUT: usize, O, F: FramebufferWithoutExtra<OUT>>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'b, '_, 'a, Self, O, F::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, F>,
    ) -> Result<(), DrawError<'b>> {
        active_shader
            .set_uniform("vertex".to_string(), &self.vertices)
            .map_err(|err| DrawError::SetUniform(err))?;

        let len = if self.vertices.len() <= LINE_SHADER_MAX_VERTICES {
            ElementArrayLen::new(self.vertices.len() as _)
        } else {
            return Err(DrawError::LineTooLong {
                len: self.vertices.len(),
            });
        };

        vertex_array::dummy_vertex_array()
            .draw_len(active_shader, active_framebuffer, len)
            .map_err(|err| DrawError::VertexArrayDraw(err))?;

        Ok(())
    }
}

impl<U: Uniform> Model for Line<U> {
    type Vertex = NoVertex;
}
