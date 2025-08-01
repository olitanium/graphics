#![feature(generic_const_exprs)]
#![expect(incomplete_features)]

mod matrix;
//mod orientation;
mod vector;

pub use matrix::Matrix;
//pub use orientation::Orientation;
pub use vector::Vector;
pub use vector::UnitVector;
