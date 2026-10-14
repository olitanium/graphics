use core::gca;

use geometry::{NoTransform, Pose};
use graphics::linear_algebra::Vector;

use crate::modelling::extended_vertex::{ExtendedVertex, IncompleteVertex};

impl ExtendedVertex for Vector<2> {
    type FromIncompleteVertexError = !;
    type PreferredTransform<'a> = NoTransform;

    const LEN: usize = gca!(2);

    fn position(&self) -> Self {
        *self
    }

    fn from_incomplete_triangle(
        triangle: [IncompleteVertex<2>; 3],
    ) -> Result<[Self; 3], Self::FromIncompleteVertexError> {
        Ok(triangle.map(|incomplete_vertex| incomplete_vertex.position))
    }
}

impl ExtendedVertex for Vector<3> {
    type FromIncompleteVertexError = !;
    type PreferredTransform<'a> = Pose;

    const LEN: usize = gca!(3);

    fn from_incomplete_triangle(
        triangle: [IncompleteVertex<3>; 3],
    ) -> Result<[Self; 3], Self::FromIncompleteVertexError> {
        Ok(triangle.map(|incomplete_vertex| incomplete_vertex.position))
    }

    fn position(&self) -> Self {
        *self
    }
}
