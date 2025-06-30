#![expect(incomplete_features)]
#![feature(array_windows)]
#![feature(array_chunks)]
#![feature(stmt_expr_attributes)]
#![feature(generic_const_exprs)]
#![feature(box_into_inner)]
#![feature(iter_chain)]
#![feature(iterator_try_collect)]
#![feature(macro_metavar_expr)]
#![feature(type_changing_struct_update)]
#![feature(iter_map_windows)]
#![feature(array_try_map)]
#![feature(string_from_utf8_lossy_owned)]
#![feature(box_as_ptr)]
#![feature(try_trait_v2)]
#![feature(ascii_char)]
#![feature(lazy_type_alias)]
#![feature(debug_closure_helpers)]
#![feature(option_array_transpose)]
#![deny(missing_debug_implementations)]
//#![deny(missing_docs)]
#![warn(clippy::complexity)]
#![warn(clippy::correctness)]
#![warn(clippy::nursery)]
#![warn(clippy::perf)]
#![warn(clippy::style)]
#![warn(clippy::suspicious)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![expect(clippy::implicit_return)]
#![expect(clippy::as_conversions)]
#![expect(clippy::float_arithmetic)]
#![expect(clippy::missing_const_for_fn)]
#![expect(clippy::must_use_candidate)]
#![expect(clippy::single_call_fn)]

mod error;
//pub use error::Error;
//pub use error::Result;

#[macro_export]
macro_rules! getter {
    ($value:ident : $type:ty) => {
        #[must_use]
        #[inline]
        pub fn $value(&self) -> &$type {
            &self.$value
        }
    };
}

#[macro_export]
macro_rules! getter_mut {
    ($value:ident : $type:ty) => {
        paste::item! {
            #[must_use]
            #[inline]
            pub fn [<$value _mut>](&mut self) -> &mut $type {
            &mut self.$value
        }
        }
    };
}

#[macro_export]
macro_rules! builder {

    ($name:ident: Option<$typ:ty>) => {
        #[must_use]
        #[inline]
        pub fn $name<X: Into<$typ>>(mut self, $name: X) -> Self {
            self.$name = Some($name.into());
            self
        }
    };

    ($name:ident: $typ:ty) => {
        #[must_use]
        #[inline]
        pub fn $name<X: Into<$typ>>(mut self, $name: X) -> Self {
            self.$name = $name.into();
            self
        }
    };
}

#[macro_export]
macro_rules! new {
    () => {
        pub fn new() -> Self {
            Self::default()
        }
    };
}

/*#[macro_export]
macro_rules! gl_call {
    ($input:stmt) => {{
        // eprintln!(stringify!($input));
        // Skip all previous errors which have been ignored
        while unsafe { gl::GetError() } != gl::NO_ERROR {}
        // perform the expression
        let output = unsafe { $input };
        // read through errors, returning Err if there are many.
        let errors: Vec<$crate::types::GLError> =
            std::iter::repeat_with(|| $crate::types::GLError(unsafe { gl::GetError() }))
                .take_while(|error| error.0 != gl::NO_ERROR)
                .collect();
        if errors.is_empty() {
            output
        } else {
            panic!("{:?}", errors)
        }
    }};

    ($input:stmt;) => {{
        // eprintln!(stringify!($input));
        // Skip all previous errors which have been ignored
        while unsafe { gl::GetError() } != gl::NO_ERROR {}
        // perform the expression
        unsafe { $input };
        // read through errors, returning Err if there are many.
        let errors: Vec<$crate::types::GLError> =
            std::iter::repeat_with(|| $crate::types::GLError(unsafe { gl::GetError() }))
                .take_while(|error| error.0 != gl::NO_ERROR)
                .collect();
        if !errors.is_empty() {
            panic!("{:?}", errors);
        };
    }};
}
*/