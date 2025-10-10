use graphics::linear_algebra::Vector;

use crate::modelling::advanced_vertex::ExtendedVertex;

impl ExtendedVertex<2> for Vector<2> {
    fn position(&self) -> Vector<2> {
        *self
    }

    fn from_incomplete_triangle(triangle: [super::IncompleteVertex<2>; 3]) -> Option<[Self; 3]> {
        Some(triangle.map(|incomplete_vertex| incomplete_vertex.position))
    }
}

impl ExtendedVertex<3> for Vector<3> {
    fn from_incomplete_triangle(triangle: [super::IncompleteVertex<3>; 3]) -> Option<[Self; 3]> {
        Some(triangle.map(|incomplete_vertex| incomplete_vertex.position))
    }

    fn position(&self) -> Vector<3> {
        *self
    }
}
