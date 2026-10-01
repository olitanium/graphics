use geometry::{NoTransform, Pose};
use graphics::linear_algebra::Vector;

use crate::modelling::extended_vertex::{ExtendedVertex, IncompleteVertex};

impl ExtendedVertex for Vector<2> {
    type PreferredTransform<'a> = NoTransform;

    type const LEN: usize = 2;

    fn position(&self) -> Self {
        *self
    }

    fn from_incomplete_triangle(triangle: [IncompleteVertex<2>; 3]) -> Option<[Self; 3]> {
        Some(triangle.map(|incomplete_vertex| incomplete_vertex.position))
    }
}

impl ExtendedVertex for Vector<3> {
    type PreferredTransform<'a> = Pose;

    type const LEN: usize = 3;

    fn from_incomplete_triangle(triangle: [IncompleteVertex<3>; 3]) -> Option<[Self; 3]> {
        Some(triangle.map(|incomplete_vertex| incomplete_vertex.position))
    }

    fn position(&self) -> Self {
        *self
    }
}
