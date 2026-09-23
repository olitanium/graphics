use core::mem;

use crate::gl_call;
use crate::types::{ElementArrayElem, ElementArrayId, ElementArrayLen, GLsizeiptr};

/// Tells OpenGL in which order the vertices of the VertexBuffer should be
/// drawn. Internally (inside GPU memory) is an array of ElementArrayElem.
/// Three ElementArrayElem in a row form one triangle primitive.
#[derive(Debug)]
pub struct ElementArrayBuffer {
    /// The OpenGL provided 'name'
    id: ElementArrayId,
    /// The length of the internal buffer.
    elements: Vec<usize>,
}

impl Drop for ElementArrayBuffer {
    fn drop(&mut self) {
        let primitive = self.id.as_primitive();
        gl_call! {
            gl::DeleteBuffers(1, &raw const primitive);
        }
    }
}

impl ElementArrayBuffer {
    /// Create new `ElementArrayBuffer` from a slice of `ElementArrayElem`s.
    /// There is no builder for this object.
    pub(crate) fn new(elements: Vec<ElementArrayElem>) -> Self {
        // let len = ElementArrayLen::new(contents.len() as GLsizei);

        let id = {
            let mut id = 0;
            gl_call! {
                gl::GenBuffers(1, &raw mut id);
            }
            ElementArrayId::new(id)
        };

        gl_call! {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id.as_primitive());
        }

        gl_call! {
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                mem::size_of_val(elements.as_slice()) as GLsizeiptr,
                elements.as_ptr().cast(),
                gl::STATIC_DRAW,
            );
        };

        let elements_usize = elements
            .into_iter()
            .map(ElementArrayElem::as_usize)
            .collect();

        Self {
            id,
            elements: elements_usize,
        }
    }

    pub(crate) fn len(&self) -> ElementArrayLen {
        ElementArrayLen::new(self.elements.len() as i32)
    }

    pub(crate) fn triangles(&self) -> &[[usize; 3]] {
        self.elements.as_chunks().0
    }

    /// Bind ElementArrayBuffer
    pub(crate) fn bind(&self) {
        gl_call! {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id.as_primitive());
        }
    }

    /// Unbind any/all ElementArrayBuffer
    pub(crate) fn unbind() {
        gl_call! {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }
}
