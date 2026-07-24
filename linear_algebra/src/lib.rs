#![expect(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(const_clone)]
#![feature(const_default)]
#![feature(const_trait_impl)]
#![feature(const_index)]
#![feature(const_slice_make_iter)]
#![feature(const_ops)]
#![feature(const_destruct)]
#![feature(const_cmp)]
#![feature(const_convert)]

mod matrix;
mod vector;

#[cfg(test)]
mod test;

pub use matrix::Matrix;
pub use vector::{CompactUnitVector, UnitVector, Vector};
