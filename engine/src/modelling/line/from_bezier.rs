use bezier::BezierCurve;
use graphics::linear_algebra::Vector;
use graphics::shader_program::Uniform;

use crate::modelling::Line;

impl<const DIMS: usize> Line<Vector<DIMS>>
where
    Vector<DIMS>: Uniform,
{
    fn from_bezier<const N: usize>(
        _curve: BezierCurve<DIMS, N>,
        _lower_bound: f32,
        _upper_bound: f32,
        _delta: f32,
    ) -> Self {
        // TODO: create array between lb and ub and get the bezier curve from it.
        todo!();
    }
}
