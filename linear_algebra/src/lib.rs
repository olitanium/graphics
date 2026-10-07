//! A comprehensive linear algrbra crate with `Vectors`, new type
//! `UnitVectors` guaranteed to have magnitude 1.0, and `Matrix`es.
#![no_std]
#![expect(incomplete_features, reason = "Nightly features used")]
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
#![feature(const_option_ops)]
#![feature(const_try)]
#![feature(const_array)]
#![feature(const_closures)]
#![feature(maybe_uninit_array_assume_init)]
#![feature(option_array_transpose)]
#![feature(stmt_expr_attributes)]
#![feature(maybe_uninit_uninit_array_transpose)]
#![feature(iter_collect_into)]
#![feature(const_bool)]

mod matrix;
mod vector;

pub use matrix::Matrix;
pub use vector::{CompactUnitVector, UnitVector, Vector};

type VectorPreferredFloat = f32;
