#![feature(generic_const_exprs)]
#![expect(incomplete_features)]

mod matrix;
mod vector;

pub use matrix::Matrix;
pub use vector::{UnitVector, Vector};
