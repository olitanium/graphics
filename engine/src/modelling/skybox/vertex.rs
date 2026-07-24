use graphics::linear_algebra::Vector;
use graphics::types::VertexAttrType;
use graphics::vertex::Vertex;

use crate::TransformBy;
use crate::modelling::advanced_vertex::{ExtendedVertex, IncompleteVertex};
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{CubicVertex, SimpleVertex};

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct SkyBoxVertex(SimpleVertex);

impl Vertex for SkyBoxVertex {
    type const ELEMENT_COUNT: usize = <SimpleVertex as Vertex>::ELEMENT_COUNT;

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        SimpleVertex::offsets()
    }

    fn types_of() -> [VertexAttrType; Self::ELEMENT_COUNT] {
        SimpleVertex::types_of()
    }
}

impl ExtendedVertex<3> for SkyBoxVertex {
    fn position(&self) -> Vector<3> {
        self.0.position()
    }

    fn from_incomplete_triangle(triangle: [IncompleteVertex<3>; 3]) -> Option<[Self; 3]> {
        SimpleVertex::from_incomplete_triangle(triangle).map(|arr| arr.map(SkyBoxVertex))
    }
}

impl TransformBy<Pose> for SkyBoxVertex {
    fn transform_by(&self, operator: &Pose) -> Self {
        Self(self.0.transform_by(operator))
    }
}

impl CubicVertex for SkyBoxVertex {}
