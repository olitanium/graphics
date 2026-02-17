mod incomplete;
use graphics::linear_algebra::{UnitVector, Vector};
use graphics::vertex::{NoVertex, Vertex};
pub use incomplete::IncompleteVertex;

mod impls;

use crate::TransformBy;
use crate::modelling::SimpleVertex;
use crate::modelling::cubic::geometry::Pose;

pub trait ExtendedVertex<const N: usize>: Vertex {
    fn position(&self) -> Vector<N>;
    fn from_incomplete_triangle(triangle: [IncompleteVertex<N>; 3]) -> Option<[Self; 3]>;
}

pub trait CubicVertex: ExtendedVertex<3> + TransformBy<Pose> {}

pub trait VertexHasTangent: CubicVertex {
    fn normal(self) -> UnitVector<3>;
    fn into_simplevertex(self) -> SimpleVertex;
}

impl<V: CubicVertex> TransformBy<Pose> for [V; 3] {
    fn transform_by(&self, operator: &Pose) -> Self {
        self.map(|vertex| vertex.transform_by(operator))
    }
}

impl ExtendedVertex<0> for NoVertex {
    fn position(&self) -> Vector<0> {
        Vector::ZERO
    }

    fn from_incomplete_triangle(_: [IncompleteVertex<0>; 3]) -> Option<[Self; 3]> {
        Some([Self; 3])
    }
}
