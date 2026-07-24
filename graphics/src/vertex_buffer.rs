use std::mem;

use crate::types::{VertexBufferId, VertexBufferLen};
use crate::vertex::Vertex;
use crate::{gl_call, types};

#[derive(Debug)]
pub struct VertexBuffer<V: Vertex> {
    id: VertexBufferId,
    vertices: Vec<V>,
}

impl<V: Vertex> VertexBuffer<V> {
    #[must_use]
    pub fn new(vertices: Vec<V>) -> Self {
        let id = {
            let mut id = 0;
            gl_call! {
                gl::GenBuffers(1, &raw mut id);
            }
            VertexBufferId::new(id)
        };

        gl_call! {
            gl::BindBuffer(gl::ARRAY_BUFFER, id.as_primitive());
        }

        gl_call! { gl::BufferData(
            gl::ARRAY_BUFFER,
            #[allow(clippy::cast_possible_wrap)]
            // slice not allowed to be bigger than isize for non-zero types anyway
            (mem::size_of_val(vertices.as_slice()) as types::GLsizeiptr),
            vertices.as_ptr().cast(),
            gl::STATIC_DRAW,
        ); };

        Self { id, vertices }
    }

    pub(crate) fn len(&self) -> VertexBufferLen {
        #[allow(clippy::cast_possible_truncation)]
        // vertices len is unlikely to be greater than 4 billion
        VertexBufferLen::new(self.vertices.len() as VertexBufferLen::Primitive)
    }

    pub(crate) fn bind(&self) {
        gl_call! {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id.as_primitive());
        }
    }

    pub(crate) fn unbind() {
        gl_call! {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    pub(crate) fn vertices(&self) -> &[V] {
        &self.vertices
    }
}

impl<V: Vertex> Drop for VertexBuffer<V> {
    fn drop(&mut self) {
        let primitive = self.id.as_primitive();
        gl_call! {
            gl::DeleteBuffers(1, &raw const primitive);
        }
    }
}
