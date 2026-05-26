use bezier::BezierCurve;
use graphics::{linear_algebra::Vector, shader_program::Uniform, vertex::Vertex};

use crate::modelling::Line;

impl<const DIMS: usize> Line<Vector<DIMS>>
where Vector<DIMS>: Uniform {
    fn from_bezier<const N: usize>(curve: BezierCurve<DIMS, N>, lower_bound: f32, upper_bound: f32, delta: f32) -> Self {
        // TODO: create array between lb and ub and get the bezier curve from it.
        Self::new()
    }
}