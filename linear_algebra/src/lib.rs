#![expect(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(derive_const)]
#![feature(const_clone)]
#![feature(const_default)]
#![feature(const_trait_impl)]

mod matrix;
mod vector;

#[cfg(test)]
mod test;

pub use matrix::Matrix;
pub use vector::{UnitVector, Vector};
