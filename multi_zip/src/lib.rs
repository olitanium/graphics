//! Zip several iterators (of the same type) and return all of them in one go in
//! one `array` (or `Vec`).

#![feature(const_array)]
#![feature(const_iter)]
#![feature(const_trait_impl)]
#![feature(const_destruct)]

mod array;
mod vec;

pub use array::MultiZip;
pub use vec::HeapMultiZip;

#[cfg(test)]
mod test;
