use std::marker::PhantomData;
use std::mem;

use crate::types::{ToPrimitive, VertexBufferId};
use crate::vertex::Vertex;
use crate::{gl_call, types};

#[derive(Debug)] // No Clone
pub struct VertexBuffer<V: Vertex> {
    id: VertexBufferId,
    _phantom: PhantomData<Vec<V>>,
}

impl<V: Vertex> VertexBuffer<V> {
    // #[deprecated]
    // pub fn new_fromstream(contents: &[f32]) -> Self {
    // let id = {
    // let mut id = 0;
    // gl_call! {
    // gl::GenBuffers(1, &raw mut id);
    // }
    // VertexBufferId::new(id)
    // };
    //
    // gl_call! {
    // gl::BindBuffer(gl::ARRAY_BUFFER, id.to_primitive());
    // }
    //
    // gl_call! { gl::BufferData(
    // gl::ARRAY_BUFFER,
    // mem::size_of_val(contents) as types::GLsizeiptr,
    // contents.as_ptr().cast(),
    // gl::STATIC_DRAW,
    // ); };
    //
    // Self { id, _phantom: PhantomData }
    // }

    pub fn new(contents: &[V]) -> Self {
        let id = {
            let mut id = 0;
            gl_call! {
                gl::GenBuffers(1, &raw mut id);
            }
            VertexBufferId::new(id)
        };

        gl_call! {
            gl::BindBuffer(gl::ARRAY_BUFFER, id.to_primitive());
        }

        gl_call! { gl::BufferData(
            gl::ARRAY_BUFFER,
            mem::size_of_val(contents) as types::GLsizeiptr,
            contents.as_ptr().cast(),
            gl::STATIC_DRAW,
        ); };

        Self {
            id,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn bind(&self) {
        gl_call! {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id.to_primitive());
        }
    }

    pub(crate) fn unbind() {
        gl_call! {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}

impl<V: Vertex> Drop for VertexBuffer<V> {
    fn drop(&mut self) {
        let primitive = self.id.to_primitive();
        gl_call! {
            gl::DeleteBuffers(1, &raw const primitive);
        }
    }
}
