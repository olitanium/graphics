use core::gca;

use geometry::{NoTransform, Pose, TransformBy};
use graphics::linear_algebra::Vector;
use graphics::types::VertexAttrType;
use graphics::vertex::Vertex;

use crate::modelling::extended_vertex::{ExtendedVertex, IncompleteVertex};
use crate::modelling::simple_vertex::FromIncompleteVertexError;
use crate::modelling::{CubicVertex, SimpleVertex};

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct SkyBoxVertex(SimpleVertex);

impl Vertex for SkyBoxVertex {
    const ELEMENT_COUNT: usize = gca!(<SimpleVertex as Vertex>::ELEMENT_COUNT);

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        SimpleVertex::offsets()
    }

    fn types_of() -> [VertexAttrType; Self::ELEMENT_COUNT] {
        SimpleVertex::types_of()
    }
}

impl ExtendedVertex for SkyBoxVertex {
    type FromIncompleteVertexError = FromIncompleteVertexError;
    type PreferredTransform<'a> = NoTransform;

    const LEN: usize = gca!(3);

    fn position(&self) -> Vector<3> {
        self.0.position()
    }

    fn from_incomplete_triangle(
        triangle: [IncompleteVertex<3>; 3],
    ) -> Result<[Self; 3], Self::FromIncompleteVertexError> {
        SimpleVertex::from_incomplete_triangle(triangle).map(|arr| arr.map(SkyBoxVertex))
    }
}

impl TransformBy<Pose> for SkyBoxVertex {
    fn transform_by(self, operator: Pose) -> Self {
        Self(self.0.transform_by(operator))
    }
}

impl CubicVertex for SkyBoxVertex {}
