use std::{iter, ptr};

use builder::Builder;

// pub use cubic_builder::CubicBuilder;
// pub use quad_builder::QuadBuilder;
use super::framebuffer::ActiveFramebuffer;
use crate::element_array_buffer::ElementArrayBuffer;
use crate::error::Result;
use crate::framebuffer::traits::FramebufferInternals;
use crate::shader_program::ActiveShaderProgram;
use crate::types::{VertexArrayId};
use crate::vertex::Vertex;
use crate::vertex_buffer::VertexBuffer;
use crate::{gl_call, types};

// mod cubic_builder;
// mod quad_builder;
mod builder;
// mod test_models;

#[derive(Debug)]
pub struct VertexArray<V: Vertex> {
    id: VertexArrayId,
    _vertex_buffer: VertexBuffer<V>,
    element_array_buffer: ElementArrayBuffer,
}

impl<V: Vertex> Drop for VertexArray<V> {
    fn drop(&mut self) {
        let primitive = self.id.to_primitive();
        gl_call! {
            gl::DeleteVertexArrays(1, &raw const primitive);
        }
    }
}

// impl VertexArray<SimpleVertex> {
// pub fn cubic_builder() -> CubicBuilder {
// CubicBuilder::new()
// }
// }
//
// impl VertexArray<QuadVertex> {
// pub fn quad_builder() -> QuadBuilder {
// QuadBuilder::new()
// }
// }

impl<V: Vertex> VertexArray<V>
where
    [(); V::ELEMENT_COUNT]:,
{
    pub fn builder() -> Builder<V> {
        Builder::new()
    }

    fn new(vertex_buffer: VertexBuffer<V>, element_array_buffer: ElementArrayBuffer) -> Self {
        let id = {
            let mut id = 0;
            gl_call! {
                gl::GenVertexArrays(1, ptr::addr_of_mut!(id));
            }
            VertexArrayId::new(id)
        };

        gl_call! {
            gl::BindVertexArray(id.to_primitive());
        }

        vertex_buffer.bind();
        element_array_buffer.bind();

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
        // offsets
        // .iter()
        // .enumerate()
        // .fold(ptr::null::<f32>(), |pointer, (index, step)| {
        // gl_call! {
        // gl::VertexAttribPointer(
        // index as types::GLuint,
        // step as types::GLint,
        // gl::FLOAT,
        // gl::FALSE,
        // vbo_stride,
        // pointer.cast(),
        // );
        // }
        // gl_call! {
        // gl::EnableVertexAttribArray(index as types::GLuint);
        // }
        //
        // pointer.wrapping_add(*step)
        // });

        // Tidy in this order
        VertexArray::<V>::unbind();
        ElementArrayBuffer::unbind();
        VertexBuffer::<V>::unbind();

        // Return object
        VertexArray {
            id,
            _vertex_buffer: vertex_buffer,
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

    pub fn draw<M, const OUT: usize, D: FramebufferInternals<OUT>>(
        &self,
        active_shader_program: &ActiveShaderProgram<'_, '_, '_, M, D::Tex, OUT>,
        _: &mut ActiveFramebuffer<'_, '_, OUT, D>,
    ) -> Result<()> {
        self.bind();
        active_shader_program.bind_textures()?;

        active_shader_program.validate()?;

        gl_call! {
            gl::DrawElements(
                gl::TRIANGLES,
                self.element_array_buffer.len().to_primitive(),
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }

        Ok(())
    }

    pub fn empty() -> Self {
        Self::new(
            VertexBuffer::new(&[]),
            // &[],
            ElementArrayBuffer::new(&[]),
        )
    }
}
