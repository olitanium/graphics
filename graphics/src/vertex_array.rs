use std::sync::LazyLock;
use std::{iter, ptr};

use builder::Builder;

// pub use cubic_builder::CubicBuilder;
// pub use quad_builder::QuadBuilder;
use super::framebuffer::ActiveFramebuffer;
use crate::element_array_buffer::ElementArrayBuffer;
use crate::error::ApiResult;
use crate::framebuffer::traits::FramebufferWithoutExtra;
use crate::shader_program::ActiveShaderProgram;
use crate::types::{VertexArrayId, VertexBufferLen};
use crate::vertex::Vertex;
use crate::vertex_buffer::VertexBuffer;
use crate::{gl_call, types};

mod builder;
mod draw_options;
// pub use draw_options::DrawOptions;

#[derive(Debug)]
pub struct VertexArray<V: Vertex> {
    id: VertexArrayId,
    vertex_buffer: Option<VertexBuffer<V>>,
    element_array_buffer: Option<ElementArrayBuffer>,
}

impl<V: Vertex> Drop for VertexArray<V> {
    fn drop(&mut self) {
        let primitive = self.id.to_primitive();
        gl_call! {
            gl::DeleteVertexArrays(1, &raw const primitive);
        }
    }
}

impl<V: Vertex> VertexArray<V>
where
    [(); V::ELEMENT_COUNT]:,
{
    pub fn builder() -> Builder<V> {
        Builder::new()
    }

    fn new(
        vertex_buffer: VertexBuffer<V>,
        element_array_buffer: Option<ElementArrayBuffer>,
    ) -> Self {
        let id = {
            let mut id = 0;
            gl_call! {
                gl::GenVertexArrays(1, &raw mut id);
            }
            VertexArrayId::new(id)
        };

        gl_call! {
            gl::BindVertexArray(id.to_primitive());
        }

        vertex_buffer.bind();
        if let Some(buffer) = &element_array_buffer {
            buffer.bind();
        };

        for (index, (offset, (type_of, count))) in
            iter::zip(V::offsets(), V::types_of()).enumerate()
        {
            gl_call! {
                gl::VertexAttribPointer(
                    index as types::GLuint,
                    count as types::GLint,
                    type_of.get_enum(),
                    gl::FALSE,
                    V::stride() as i32,
                    (offset as *const ()).cast(),
                );
            }
            gl_call! {
                gl::EnableVertexAttribArray(index as types::GLuint);
            }
        }

        // Tidy in this order
        VertexArray::<V>::unbind();
        ElementArrayBuffer::unbind();
        VertexBuffer::<V>::unbind();

        // Return object
        VertexArray {
            id,
            vertex_buffer: Some(vertex_buffer),
            element_array_buffer,
        }
    }

    pub(crate) fn bind(&self) {
        gl_call! {
            gl::BindVertexArray(self.id.to_primitive());
        }
    }

    pub(crate) fn unbind() {
        gl_call! {
            gl::BindVertexArray(0);
        }
    }

    pub fn draw<NEVER, M, const OUT: usize, D: FramebufferWithoutExtra<OUT>>(
        &self,
        active_shader_program: &mut ActiveShaderProgram<'_, '_, '_, M, D::Tex, OUT>,
        _: &mut ActiveFramebuffer<'_, '_, OUT, D>,
    ) -> ApiResult<(), NEVER> {
        self.bind();
        active_shader_program.bind_textures()?;

        active_shader_program.validate()?;

        if let Some(element_buffer) = &self.element_array_buffer {
            gl_call! {
                gl::DrawElements(
                    gl::TRIANGLES,
                    element_buffer.len().to_primitive(),
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
            }
        } else if let Some(vbo) = self.vertex_buffer.as_ref() {
            gl_call! {
                gl::DrawArrays(gl::TRIANGLES, 0, vbo.len().to_primitive());
            }
        }

        Ok(())
    }

    pub fn empty() -> Self {
        let id = {
            let mut id = 0;
            gl_call! {
                gl::GenVertexArrays(1, &raw mut id);
            }
            VertexArrayId::new(id)
        };

        Self {
            id,
            vertex_buffer: None,
            element_array_buffer: None,
        }
    }
}

pub fn dummy_vertex_array() -> &'static VertexArray<()> {
    static DUMMY_VERTEX_ARRAY: LazyLock<VertexArray<()>> = LazyLock::new(|| VertexArray::empty());
    &DUMMY_VERTEX_ARRAY
}

impl VertexArray<()> {
    pub fn draw_len<NEVER, M, const OUT: usize, D: FramebufferWithoutExtra<OUT>>(
        &self,
        active_shader_program: &mut ActiveShaderProgram<'_, '_, '_, M, D::Tex, OUT>,
        _: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        len: VertexBufferLen,
    ) -> ApiResult<(), NEVER> {
        self.bind();
        active_shader_program.bind_textures()?;
        active_shader_program.validate()?;

        gl_call! {
            gl::DrawArrays(gl::LINE_STRIP, 0, len.to_primitive());
        }

        Ok(())
    }
}
