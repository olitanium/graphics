mod incomplete;
use graphics::linear_algebra::Vector;
use graphics::vertex::Vertex;
pub use incomplete::IncompleteVertex;

mod impls;

use crate::TransformBy;
use crate::modelling::cubic::geometry::Pose;

pub trait ExtendedVertex<const N: usize>: Vertex {
    fn position(&self) -> Vector<N>;
}

pub trait CubicVertex: ExtendedVertex<3> + TransformBy<Pose> {
    fn from_incomplete_triangle(triangle: [IncompleteVertex; 3]) -> Option<[Self; 3]>;
    //fn transform_by_pose(&self, pose: Pose) -> Self;
}

impl<V: CubicVertex> TransformBy<Pose> for [V; 3] {
    fn transform_by(&self, operator: &Pose) -> Self {
        self.map(|vertex| vertex.transform_by(operator))
    }
}