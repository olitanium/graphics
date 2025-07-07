use std::mem;

use utils::getter;

use crate::gl_call;
use crate::types::{
    ElementArrayElem,
    ElementArrayId,
    ElementArrayLen,
    GLsizei,
    GLsizeiptr,
    };

/// Tells OpenGL in which order the vertices of the VertexBuffer should be
/// drawn. Internally (inside GPU memory) is an array of ElementArrayElem.
/// Three ElementArrayElem in a row form one triangle primitive.
#[derive(Debug)]
pub struct ElementArrayBuffer {
    /// The OpenGL provided 'name'
    id: ElementArrayId,
    /// The length of the internal buffer.
    len: ElementArrayLen,
}

impl Drop for ElementArrayBuffer {
    fn drop(&mut self) {
        let primitive = self.id.to_primitive();
        gl_call! {
            gl::DeleteBuffers(1, &raw const primitive);
        }
    }
}

impl ElementArrayBuffer {
    getter!(len: ElementArrayLen);

    /// Create new `ElementArrayBuffer` from a slice of `ElementArrayElem`s.
    /// There is no builder for this object.
    pub(crate) fn new(contents: &[ElementArrayElem]) -> Self {
        let len = ElementArrayLen::new(contents.len() as GLsizei);

        let id = {
            let mut id = 0;
            gl_call! {
                gl::GenBuffers(1, &raw mut id);
            }
            ElementArrayId::new(id)
        };

        gl_call! {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id.to_primitive());
        }

        gl_call! {
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                mem::size_of_val(contents) as GLsizeiptr,
                contents.as_ptr().cast(),
                gl::STATIC_DRAW,
            );
        };

        Self { id, len }
    }

    /// Bind ElementArrayBuffer
    pub(crate) fn bind(&self) {
        gl_call! {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id.to_primitive());
        }
    }

    /// Unbind any/all ElementArrayBuffer
    pub(crate) fn unbind() {
        gl_call! {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }
}
