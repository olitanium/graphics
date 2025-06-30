#![expect(incomplete_features)]
#![feature(debug_closure_helpers)]
#![feature(string_from_utf8_lossy_owned)]
#![feature(generic_const_exprs)]
#![feature(stmt_expr_attributes)]
#![feature(type_changing_struct_update)]
pub mod texture;
pub mod buffers;
pub mod types;
pub mod shader_program;
pub mod error;


pub use buffers::Framebuffer;
pub use buffers::DefaultFramebuffer;
pub use buffers::framebuffer_builder;

#[macro_export]
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
