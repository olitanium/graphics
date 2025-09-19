use graphics::linear_algebra::Vector;
use graphics::vertex::Vertex;

use crate::modelling::advanced_vertex::ExtendedVertex;

impl ExtendedVertex<2> for Vector<2> {
    fn position(&self) -> Vector<2> {
        *self
    }
}
