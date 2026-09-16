//! A utility crate. Modules in this crate really ought to be spun off into
//! smaller manageable crates.
#![warn(missing_docs)]
#![feature(const_trait_impl)]
#![feature(const_ops)]
mod error;

mod ref_or_owned;

pub use ref_or_owned::RefOwned;

/// Testbed is a module for trying out MVP ideas before fleshing out.
pub mod addassign;

pub mod hash_match;

pub mod binary_search;

pub mod mem;

/// A small macro to make the `Builder` pattern easier to implement.
/// # Example
///
/// ```
/// # struct Builder {
/// #    value: u32,
/// #    maybe_value: Option<i32>,
/// #    other: Option<usize>,
/// # }
/// # use utils::builder;
/// impl Builder {
///     builder!(value: u32);
///     builder!(maybe_value: Option<i32>);
///     builder!(other, optional_other: Option<usize>);
/// }
/// ```
/// will expand to:
/// ```
/// # struct Builder {
/// #    value: u32,
/// #    maybe_value: Option<i32>,
/// #    other: Option<usize>,
/// # }
/// # use utils::builder;
/// impl Builder {
///     pub fn value(mut self, value: u32) -> Self {
///         self.value = value;
///         self
///     }
///
///     pub fn maybe_value(mut self, maybe_value: i32) -> Self {
///         self.maybe_value = Some(maybe_value);
///         self
///     }
///
///     pub fn other(mut self, other: usize) -> Self {
///         self.other = Some(other);
///         self
///     }
///
///     pub fn optional_other(mut self, optional_other: Option<usize>) -> Self {
///         self.other = optional_other;
///         self
///     }
/// }
/// ```
#[macro_export]
macro_rules! builder {
    ($name:ident, $optname:ident : Option < $typ:ty >) => {
        #[doc = concat!("set ", stringify!($name), " to a value.")]
        pub fn $name(mut self, $name: $typ) -> Self {
            self.$name = Some($name);
            self
        }

        #[doc = concat!("set ", stringify!($name), " to an optional value.")]
        pub fn $optname(mut self, $optname: Option<$typ>) -> Self {
            self.$name = $optname;
            self
        }
    };

    ($name:ident : Option < $typ:ty >) => {
        #[doc = concat!("set ", stringify!($name), " to an optional value.")]
        pub fn $name(mut self, $name: $typ) -> Self {
            self.$name = Some($name);
            self
        }
    };

    ($name:ident : $typ:ty) => {
        #[doc = concat!("set ", stringify!($name), " to an value.")]
        pub fn $name(mut self, $name: $typ) -> Self {
            self.$name = $name;
            self
        }
    };
}

/// A small macro to make the `Builder` pattern easier to implement.
/// creates a `new` function which uses the `Default` implementation.
#[macro_export]
macro_rules! new {
    () => {
        /// Create a new (default initialised) builder.
        pub fn new() -> Self {
            Self::default()
        }
    };
}
