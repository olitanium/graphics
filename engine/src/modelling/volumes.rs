use graphics::linear_algebra::{Vector, vector};

use crate::modelling::extended_vertex::IncompleteVertex;
use crate::modelling::{ExtendedVertex, test_models};

pub trait Volume: ExtendedVertex {
    fn volume(
        lower_bound: Self,
        upper_bound: Self,
    ) -> impl Iterator<Item = [IncompleteVertex<{ Self::LEN }>; 3]>;

    fn infinity() -> (Self, Self);
}

const UPPER_BOUND: f32 = 1_000_000.0;
const LOWER_BOUND: f32 = -UPPER_BOUND;

impl Volume for Vector<2> {
    fn volume(
        lower_bound: Self,
        upper_bound: Self,
    ) -> impl Iterator<Item = [IncompleteVertex<2>; 3]> {
        test_models::quad_triangles(lower_bound, upper_bound)
    }

    fn infinity() -> (Self, Self) {
        const {
            let upper = vector![UPPER_BOUND, UPPER_BOUND];
            let lower = vector![LOWER_BOUND, LOWER_BOUND];
            (upper, lower)
        }
    }
}

impl Volume for Vector<3> {
    fn volume(
        lower_bound: Self,
        upper_bound: Self,
    ) -> impl Iterator<Item = [IncompleteVertex<3>; 3]> {
        test_models::cube_triangles(lower_bound, upper_bound)
    }

    fn infinity() -> (Self, Self) {
        const {
            let upper = vector![UPPER_BOUND, UPPER_BOUND, UPPER_BOUND];
            let lower = vector![LOWER_BOUND, LOWER_BOUND, LOWER_BOUND];
            (upper, lower)
        }
    }
}
