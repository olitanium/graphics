mod incomplete;

use core::error::Error;
use std::gca;

use graphics::linear_algebra::{UnitVector, Vector};
use graphics::vertex::{NoVertex, Vertex};
pub use incomplete::IncompleteVertex;

mod impls;

use geometry::{NoTransform, Transform};

use crate::modelling::SimpleVertex;

pub trait ExtendedVertex: Vertex {
    #[rustc_always_gca]
    const LEN: usize;
    type PreferredTransform<'a>: Transform<Self> + Clone;
    type FromIncompleteVertexError: Error;

    fn position(&self) -> Vector<gca!(Self::LEN)>;

    fn multi_position(vec: &[Self; 3]) -> [Vector<gca!(Self::LEN)>; 3] {
        vec.each_ref().map(Self::position)
    }

    fn from_incomplete_triangle(
        triangle: [IncompleteVertex<gca! {Self::LEN }>; 3],
    ) -> Result<[Self; 3], Self::FromIncompleteVertexError>;
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
    type FromIncompleteVertexError = !;
    type PreferredTransform<'a> = NoTransform;

    const LEN: usize = gca!(0);

    fn position(&self) -> Vector<0> {
        Vector::ZERO
    }

    fn from_incomplete_triangle(
        _: [IncompleteVertex<0>; 3],
    ) -> Result<[Self; 3], Self::FromIncompleteVertexError> {
        Ok([Self; 3])
    }
}
