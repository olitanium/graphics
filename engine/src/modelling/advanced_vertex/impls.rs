use graphics::linear_algebra::Vector;

use crate::modelling::advanced_vertex::ExtendedVertex;

impl ExtendedVertex<2> for Vector<2> {
    fn position(&self) -> Vector<2> {
        *self
    }
}
