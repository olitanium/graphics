mod incomplete;
use graphics::linear_algebra::Vector;
use graphics::vertex::Vertex;
pub use incomplete::IncompleteVertex;

mod impls;

use crate::modelling::cubic::geometry::Pose;

pub trait ExtendedVertex<const N: usize>: Vertex {
    fn position(&self) -> Vector<N>;
}

pub trait CubicVertex: ExtendedVertex<3> {
    fn from_incomplete_triangle(triangle: [IncompleteVertex; 3]) -> [Self; 3];
    fn transform_by_pose(&self, pose: Pose) -> Self;
}
