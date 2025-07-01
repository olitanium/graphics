use super::VertexArray;
use crate::buffers::element_array_buffer::ElementArrayBuffer;
use crate::buffers::vertex_buffer::{Vertex, VertexBuffer};
use linear_algebra::{UnitVector, Vector};
use crate::types::ElementArrayElem;
use utils::{builder, new};

#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct IncompleteVertex {
    pub position: Vector<3>,
    pub texture: Vector<2>,
    pub normal: Option<UnitVector<3>>,
    pub tangent: Option<UnitVector<3>>,
}

impl IncompleteVertex {
    pub fn new(position: Vector<3>, texture: Vector<2>) -> Self {
        Self { position, texture, ..Default::default() }
    }

    builder!(normal, opt_normal: Option<UnitVector<3>>);
    builder!(tangent, opt_tangent: Option<UnitVector<3>>);
}

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
        /*
        let new_normal = if incomplete_triangle.iter().all(|v| v.normal.is_none()) {
            let ab = incomplete_triangle[1].position - incomplete_triangle[0].position;
            let ac = incomplete_triangle[2].position - incomplete_triangle[0].position;
            ab.cross(ac).normalize()
        } else {
            UnitVector::new_unchecked([0.0, 0.0, 1.0])
        };

        let new_tangent = if incomplete_triangle.iter().all(|v| v.tangent.is_none()) {
            let ab = incomplete_triangle[1].position - incomplete_triangle[0].position;
            let ac = incomplete_triangle[2].position - incomplete_triangle[0].position;
            let z = incomplete_triangle[1].texture - incomplete_triangle[0].texture;
            let g = incomplete_triangle[2].texture - incomplete_triangle[0].texture;

            (ab.scale(g[1]) - ac.scale(z[1])) // .scale(1.0/(g[1]*z[0] - g[0]*z[1]))
                .normalize() // change of basis
        } else {
            UnitVector::new_unchecked([0.0, 0.0, 1.0])
        };

        let complete_triangle = incomplete_triangle.map(
            |IncompleteVertex {
                 position,
                 texture,
                 normal,
                 tangent,
             }| SimpleVertex {
                position,
                texture,
                normal: normal.unwrap_or(new_normal),
                tangent: tangent.unwrap_or(new_tangent),
            },
        );

        self.push_triangle(complete_triangle)*/
    }
}
