#![feature(generic_const_exprs)]
#![expect(incomplete_features)]

mod matrix;
mod vector;

#[cfg(test)]
mod test;

pub use matrix::Matrix;
pub use vector::{UnitVector, Vector};
