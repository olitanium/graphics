//! Floating point types which are limited to `-1..1` or `0..1`.

#![feature(signed_bigint_helpers)]
#![feature(const_unsigned_bigint_helpers)]
#![feature(const_ops)]
#![feature(const_trait_impl)]
#![feature(const_convert)]

mod sf;
mod uf;

pub use sf::{SignedUnitf8, SignedUnitf16};
pub use uf::{Unitf8, Unitf16};
