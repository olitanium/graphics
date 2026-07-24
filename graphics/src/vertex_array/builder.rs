use utils::{builder, new};

use super::VertexArray;
use crate::element_array_buffer::ElementArrayBuffer;
use crate::types::ElementArrayElem;
use crate::vertex::Vertex;
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
        #[allow(clippy::cast_possible_truncation)]
        // vertices are not likely to have length greater than GLuint::MAX
        let original_len = self.vertices.len() as crate::types::GLuint;
        self.vertices.extend(triangle);
        self.element_array
            .extend((original_len..=original_len + 2).map(ElementArrayElem::new));
    }

    pub fn push_triangle_by_index(&mut self, triangle: [ElementArrayElem; 3]) {
        self.element_array.extend(triangle);
    }

    pub fn build(self) -> VertexArray<V> {
        let vertex_buffer = VertexBuffer::new(self.vertices);
        let element_array_buffer = ElementArrayBuffer::new(self.element_array);

        VertexArray::new(vertex_buffer, Some(element_array_buffer))
    }

    pub fn extend_triangles<I: IntoIterator<Item = [V; 3]>>(&mut self, iter: I) {
        for triangle in iter {
            self.push_triangle(triangle);
        }
    }
}
