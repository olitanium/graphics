use alloc::borrow::Cow;
use core::{ffi, iter, ptr};
use std::sync::LazyLock;

use builder::Builder;
use utils::error_boilerplate;

use super::framebuffer::ActiveFramebuffer;
use crate::element_array_buffer::ElementArrayBuffer;
use crate::framebuffer::traits::FramebufferWithoutExtra;
use crate::shader_program::{ActiveShaderProgram, BindTexturesError, ValidateError};
use crate::types::{ElementArrayLen, VertexArrayId, VertexAttrType};
use crate::vertex::{NoVertex, Vertex};
use crate::vertex_buffer::VertexBuffer;
use crate::{Model, gl_call, types};

mod builder;

pub type VertexArray<V: Vertex> = GenVertexArray<VertexBuffer<V>>;

#[derive(Debug)]
pub struct GenVertexArray<B> {
    id: VertexArrayId,
    vertex_buffer: B,
    element_array_buffer: Option<ElementArrayBuffer>,
}

impl<B> Drop for GenVertexArray<B> {
    fn drop(&mut self) {
        let primitive = self.id.as_primitive();
        gl_call! {
            gl::DeleteVertexArrays(1, &raw const primitive);
        }
    }
}

impl<V: Vertex> GenVertexArray<VertexBuffer<V>> {
    #[must_use = "builders must be built to obtain their relevant type"]
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
            gl::BindVertexArray(id.as_primitive());
        }

        vertex_buffer.bind();
        if let Some(buffer) = &element_array_buffer {
            buffer.bind();
        }

        for (index, (offset, type_of)) in iter::zip(V::offsets(), V::types_of()).enumerate() {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "there probably will not be more than GLuint::MAX components to a \
                          vertex, but usize is the `enumerate` type"
            )]
            let index = index as types::GLuint;

            #[expect(
                clippy::cast_possible_truncation,
                reason = "there probably will not be more than GLsizei::MAX bytes in a vertex but \
                          usize is a more \"Rust\" type"
            )]
            #[expect(
                clippy::cast_possible_wrap,
                reason = "there probably will not be more than GLsizei::MAX bytes in a vertex but \
                          usize is a more \"Rust\" type"
            )]
            let stride = V::stride() as types::GLsizei;
            match type_of {
                VertexAttrType::GLfloat(count) => gl_call! {
                    gl::VertexAttribPointer(
                        index,
                        types::GLint::from(count),
                        type_of.get_enum(),
                        gl::FALSE,
                        stride,
                        offset as *const ffi::c_void,
                    );
                },
                VertexAttrType::GLint(count) => gl_call! {
                    gl::VertexAttribIPointer(
                        index,
                        types::GLint::from(count),
                        type_of.get_enum(),
                        stride,
                        offset as *const ffi::c_void,
                    );
                },
            }

            gl_call! {
                gl::EnableVertexAttribArray(index);
            }
        }

        // Tidy in this order
        Self::unbind();
        ElementArrayBuffer::unbind();
        VertexBuffer::<V>::unbind();

        // Return object
        Self {
            id,
            vertex_buffer,
            element_array_buffer,
        }
    }

    #[must_use]
    pub fn triangles(&self) -> Cow<'_, [[V; 3]]> {
        self.element_array_buffer.as_ref().map_or_else(
            || Cow::Borrowed(self.vertex_buffer.vertices().as_chunks().0),
            |element| {
                let triangles = element.triangles();
                let vertices = self.vertex_buffer.vertices();

                let option_vec_triangles: Option<Vec<_>> = triangles
                    .iter()
                    .map(|index| index.map(|value| vertices.get(value).copied()).transpose())
                    .collect();

                Cow::Owned(option_vec_triangles.unwrap_or_default())
            },
        )
    }
}

impl<B> GenVertexArray<B> {
    pub(crate) fn bind(&self) {
        gl_call! {
            gl::BindVertexArray(self.id.as_primitive());
        }
    }

    pub(crate) fn unbind() {
        gl_call! {
            gl::BindVertexArray(0);
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum VertexArrayDrawError<'a> {
    BindTextures(BindTexturesError<'a>),
    Validate(ValidateError),
}
error_boilerplate!(<'a> VertexArrayDrawError<'a>);

impl<V: Vertex> GenVertexArray<VertexBuffer<V>> {
    /// # Errors
    ///
    /// Errors when the active shader program is not correctly configured for
    /// drawing. This may be because the shader program as written does not line
    /// up with the type provided to Rust.
    pub fn draw<'a, M, O, const OUT: usize, D>(
        &self,
        active_shader_program: &mut ActiveShaderProgram<'a, '_, '_, M, O, D::Tex, OUT>,
        _: &mut ActiveFramebuffer<'_, '_, OUT, D>,
    ) -> Result<(), VertexArrayDrawError<'a>>
    where
        M: Model<Vertex = V>,
        D: FramebufferWithoutExtra<OUT>,
    {
        self.bind();
        let texture_result = active_shader_program
            .bind_textures()
            .map_err(VertexArrayDrawError::BindTextures);

        // match texture_result {
        // Ok(()) => {}
        // Err(GenVertexArrayDrawError::BindTextures(BindTexturesError::SetUniform(
        // SetUniformError::GetLocation(GetUniformLocationError::NotFound {
        // c_name, .. }), ))) if
        // c_name.to_string_lossy().contains("texture") => { /* do
        // nothing */ } Err(x) => return Err(x),
        // }

        active_shader_program
            .validate()
            .map_err(VertexArrayDrawError::Validate)?;

        if let Some(element_buffer) = &self.element_array_buffer {
            element_buffer.bind();
            gl_call! {
                gl::DrawElements(
                    gl::TRIANGLES,
                    element_buffer.len().as_primitive(),
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
            }
        } else {
            gl_call! {
                gl::DrawArrays(gl::TRIANGLES, 0, self.vertex_buffer.len().as_primitive());
            }
        }

        texture_result
    }
}

fn empty() -> GenVertexArray<()> {
    let id = {
        let mut id = 0;
        gl_call! {
            gl::GenVertexArrays(1, &raw mut id);
        }
        VertexArrayId::new(id)
    };

    GenVertexArray {
        id,
        vertex_buffer: (),
        element_array_buffer: None,
    }
}

#[must_use]
pub fn dummy_vertex_array() -> &'static GenVertexArray<()> {
    static DUMMY_VERTEX_ARRAY: LazyLock<GenVertexArray<()>> = LazyLock::new(empty);
    &DUMMY_VERTEX_ARRAY
}

impl GenVertexArray<()> {
    /// # Errors
    ///
    /// Errors when the active shader program is not correctly configured for
    /// drawing. This may be because the shader program as written does not line
    /// up with the type provided to Rust.
    pub fn draw_len<'a, M, O, const OUT: usize, D>(
        &self,
        active_shader_program: &mut ActiveShaderProgram<'a, '_, '_, M, O, D::Tex, OUT>,
        _: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        len: ElementArrayLen,
    ) -> Result<(), VertexArrayDrawError<'a>>
    where
        M: Model<Vertex = NoVertex>,
        D: FramebufferWithoutExtra<OUT>,
    {
        self.bind();

        #[cfg(debug_assertions)]
        active_shader_program
            .validate()
            .map_err(VertexArrayDrawError::Validate)?;

        gl_call! {
            gl::DrawArrays(gl::LINE_STRIP, 0, len.as_primitive());
        }

        Ok(())
    }
}
