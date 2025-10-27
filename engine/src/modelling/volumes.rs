use graphics::linear_algebra::Vector;

use crate::modelling::advanced_vertex::IncompleteVertex;
use crate::modelling::{ExtendedVertex, test_models};

pub trait Volume<const N: usize>: ExtendedVertex<N> {
    fn volume(
        lower_bound: Self,
        upper_bound: Self,
    ) -> impl Iterator<Item = [IncompleteVertex<N>; 3]>;
}

impl Volume<2> for Vector<2> {
    fn volume(
        lower_bound: Self,
        upper_bound: Self,
    ) -> impl Iterator<Item = [IncompleteVertex<2>; 3]> {
        test_models::quad_triangles(lower_bound, upper_bound)
    }
}

impl Volume<3> for Vector<3> {
    fn volume(
        lower_bound: Self,
        upper_bound: Self,
    ) -> impl Iterator<Item = [IncompleteVertex<3>; 3]> {
        test_models::cube_triangles(lower_bound, upper_bound)
    }
}
