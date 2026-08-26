use graphics::linear_algebra::Vector;

use crate::modelling::extended_vertex::IncompleteVertex;
use crate::modelling::{ExtendedVertex, test_models};

pub trait Volume: ExtendedVertex {
    fn volume(
        lower_bound: Self,
        upper_bound: Self,
    ) -> impl Iterator<Item = [IncompleteVertex<{ Self::LEN }>; 3]>;
}

impl Volume for Vector<2> {
    fn volume(
        lower_bound: Self,
        upper_bound: Self,
    ) -> impl Iterator<Item = [IncompleteVertex<2>; 3]> {
        test_models::quad_triangles(lower_bound, upper_bound)
    }
}

impl Volume for Vector<3> {
    fn volume(
        lower_bound: Self,
        upper_bound: Self,
    ) -> impl Iterator<Item = [IncompleteVertex<3>; 3]> {
        test_models::cube_triangles(lower_bound, upper_bound)
    }
}
