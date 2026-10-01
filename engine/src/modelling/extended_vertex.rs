mod incomplete;

use graphics::linear_algebra::{UnitVector, Vector};
use graphics::vertex::{NoVertex, Vertex};
pub use incomplete::IncompleteVertex;

mod impls;

use geometry::{NoTransform, Transform};

use crate::modelling::SimpleVertex;

pub trait ExtendedVertex: Vertex {
    type const LEN: usize;
    type PreferredTransform<'a>: Transform<Self> + Clone;

    fn position(&self) -> Vector<{ Self::LEN }>;

    fn multi_position(vec: &[Self; 3]) -> [Vector<{ Self::LEN }>; 3] {
        vec.each_ref().map(Self::position)
    }

    fn from_incomplete_triangle(
        triangle: [IncompleteVertex<{ Self::LEN }>; 3],
    ) -> Option<[Self; 3]>;
}

pub trait CubicVertex: ExtendedVertex<LEN = 3> {
    #[inline]
    fn triangle_normal(triangle: &[Self; 3]) -> UnitVector<3> {
        let [posa, posb, posc] = triangle.each_ref().map(Self::position);
        (posb - posa)
            .cross(posc - posa)
            .try_normalize()
            .unwrap_or_default()
    }

    fn multi_normal(triangle: &[Self; 3]) -> [UnitVector<3>; 3] {
        let vec = Self::triangle_normal(triangle);
        [vec; 3]
    }
}

pub trait VertexHasTangent: CubicVertex {
    #[inline]
    fn triangle_normal_tangent(triangle: &[Self; 3]) -> UnitVector<3> {
        let sum_vector: Option<Vector<_>> = triangle.iter().map(|vert| vert.normal().v()).sum();
        // if let Some(unit) = sum_vector.and_then(|vec| vec.try_normalize()) {
        // unit
        // } else {
        // Self::triangle_normal(triangle)
        // }

        sum_vector
            .and_then(Vector::try_normalize)
            .unwrap_or_else(|| Self::triangle_normal(triangle))
    }

    fn triangle_multi_normal_tangent(triangle: &[Self; 3]) -> [UnitVector<3>; 3] {
        triangle.each_ref().map(Self::normal)
    }

    fn normal(&self) -> UnitVector<3>;
    fn into_simplevertex(self) -> SimpleVertex;
}

impl ExtendedVertex for NoVertex {
    type PreferredTransform<'a> = NoTransform;

    type const LEN: usize = 0;

    fn position(&self) -> Vector<0> {
        Vector::ZERO
    }

    fn from_incomplete_triangle(_: [IncompleteVertex<0>; 3]) -> Option<[Self; 3]> {
        Some([Self; 3])
    }
}
