//! Zip several iterators (of the same type) and return all of them in one go in
//! one `array` (or `Vec`).

mod array;
mod vec;

pub use array::MultiZip;
pub use vec::HeapMultiZip;

#[cfg(test)]
mod test;
