use utils::{builder, new};

use super::VertexArray;
use crate::element_array_buffer::ElementArrayBuffer;
use crate::types::ElementArrayElem;
use crate::vertex::{IncompleteVertex, Vertex};
use crate::vertex_buffer::VertexBuffer;

#[derive(Debug, Clone)]
pub struct Builder<V: Vertex> {
    vertices: Vec<V>,
    element_array: Vec<ElementArrayElem>,
}

impl<V: Vertex> Default for Builder<V> {
    fn default() -> Self {
        Self {
            vertices: Vec::default(),
            element_array: Vec::default(),
        }
    }
}

impl<V: Vertex> Builder<V>
where
    [(); V::ELEMENT_COUNT]:,
{
    new!();

    builder!(vertices: Vec<V>);

    builder!(element_array: Vec<ElementArrayElem>);

    pub fn push_triangle(&mut self, triangle: [V; 3]) {
        let original_len = self.vertices.len() as crate::types::GLuint;
        self.vertices.extend(triangle);
        self.element_array
            .extend([original_len, original_len + 1, original_len + 2].map(ElementArrayElem::new));
    }

    pub fn push_triangle_by_index(&mut self, triangle: [ElementArrayElem; 3]) {
        self.element_array.extend(triangle);
    }

    pub fn build(self) -> VertexArray<V> {
        let vertex_buffer = VertexBuffer::new(&self.vertices);
        let element_array_buffer = ElementArrayBuffer::new(&self.element_array);
        VertexArray::new(vertex_buffer, element_array_buffer)
    }

    pub fn push_incomplete_triangle(&mut self, incomplete_triangle: &[IncompleteVertex; 3]) {
        let complete_triangle = V::from_incomplete_triangle(incomplete_triangle);
        self.push_triangle(complete_triangle);
    }
}
