use crate::vertex::Vertex;
use crate::vertex_array::VertexArray;

pub trait Model {
    type Vertex: Vertex;
}

impl<V: Vertex> Model for VertexArray<V> {
    type Vertex = V;
}
